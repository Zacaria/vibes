use std::{convert::Infallible, net::TcpListener, sync::Arc};

use anyhow::{Context, Result};
use futures::future::BoxFuture;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use open::that;
use tokio::sync::oneshot;

type TokenSender = oneshot::Sender<(String, String)>;
type SharedTokenSender = Arc<parking_lot::Mutex<Option<TokenSender>>>;

use crate::domain::Session;

use super::{client::SupabaseClient, session::SessionStore, SupabaseConfig};

pub async fn run_passkey_flow(
    config: &SupabaseConfig,
    client: &SupabaseClient,
    store: &SessionStore,
) -> Result<Session> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).context("bind loopback for passkey")?;
    let addr = listener.local_addr()?;
    let redirect = format!("http://{}:{}/callback", addr.ip(), addr.port());
    let auth_url = format!(
        "{}/auth/passkey?redirect_to={}",
        config.url.trim_end_matches('/'),
        urlencoding::Encoded::new(&redirect)
    );

    let (token_tx, token_rx) = oneshot::channel();
    let token_tx: SharedTokenSender = Arc::new(parking_lot::Mutex::new(Some(token_tx)));
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let make_svc = make_service_fn(move |_| {
        let tx = token_tx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                handle_request(req, tx.clone())
            }))
        }
    });

    let server = Server::from_tcp(listener)
        .context("build passkey server")?
        .serve(make_svc)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });

    tokio::spawn(async move {
        if let Err(err) = server.await {
            tracing::error!(error = %err, "passkey server error");
        }
    });

    that(auth_url).context("open browser for passkey")?;

    let (_access, refresh) = token_rx.await.context("await passkey callback")?;
    shutdown_tx.send(()).ok();
    let session = client
        .refresh_session(&refresh)
        .await
        .context("exchange passkey refresh token")?;
    store.save(&session)?;
    Ok(session)
}

fn handle_request(
    req: Request<Body>,
    tx: SharedTokenSender,
) -> BoxFuture<'static, Result<Response<Body>, Infallible>> {
    Box::pin(async move {
        if req.uri().path() != "/callback" {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap());
        }
        let query = req.uri().query().unwrap_or("");
        let params: Vec<(&str, &str)> = query
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some(k), Some(v)) => Some((k, v)),
                    _ => None,
                }
            })
            .collect();
        let mut access = None;
        let mut refresh = None;
        for (k, v) in params {
            if k == "access_token" {
                access = Some(v.to_string());
            }
            if k == "refresh_token" {
                refresh = Some(v.to_string());
            }
        }
        if let (Some(access), Some(refresh)) = (access, refresh) {
            if let Some(sender) = tx.lock().take() {
                sender.send((access.clone(), refresh.clone())).ok();
            }
            let body = Body::from("<html><body><h1>Passkey login successful</h1>You may close this window.</body></html>");
            Ok(Response::new(body))
        } else {
            Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Missing tokens"))
                .unwrap())
        }
    })
}

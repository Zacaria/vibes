use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use tokio::time::sleep;
use tracing::instrument;

use crate::domain::{AudienceScope, FeedFilter, Post, Session};

use super::auth;
use super::session_store::SessionStore;

#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
}

impl SupabaseConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let url = std::env::var("SUPABASE_URL").context("SUPABASE_URL missing")?;
        let anon_key = std::env::var("SUPABASE_ANON_KEY").context("SUPABASE_ANON_KEY missing")?;
        Ok(Self { url, anon_key })
    }
}

pub struct SupabaseClient {
    pub client: Client,
    pub cfg: SupabaseConfig,
    pub sessions: SessionStore,
}

impl SupabaseClient {
    pub fn new(cfg: SupabaseConfig, sessions: SessionStore) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("cli-twitter/0.1")
            .build()?;
        Ok(Self {
            client,
            cfg,
            sessions,
        })
    }

    pub async fn login_email(&self, email: &str, password: &str) -> Result<Session> {
        let session = auth::login_with_email(&self.client, &self.cfg, email, password).await?;
        self.sessions.save(&session).await?;
        Ok(session)
    }

    pub async fn restore_session(&self) -> Result<Option<Session>> {
        self.sessions.load().await
    }

    pub async fn ensure_session(&self) -> Result<Option<Session>> {
        if let Some(mut session) = self.sessions.load().await? {
            if session.tokens.is_expired() {
                let tokens =
                    auth::refresh_session(&self.client, &self.cfg, &session.tokens.refresh_token)
                        .await?;
                session.tokens = tokens;
                self.sessions.save(&session).await?;
            }
            return Ok(Some(session));
        }
        Ok(None)
    }

    #[instrument(skip_all, fields(filter = ?filter))]
    pub async fn fetch_feed(&self, session: &Session, filter: FeedFilter) -> Result<Vec<Post>> {
        retry_async(|| auth::fetch_feed(&self.client, &self.cfg, session, filter)).await
    }

    #[instrument(skip_all, fields(scope = %scope))]
    pub async fn create_post(
        &self,
        session: &Session,
        text: &str,
        scope: AudienceScope,
    ) -> Result<Post> {
        retry_async(|| auth::post_message(&self.client, &self.cfg, session, text, scope)).await
    }
}

async fn retry_async<F, Fut, T>(mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(res) => return Ok(res),
            Err(err) if attempts < 3 => {
                attempts += 1;
                let backoff = Duration::from_millis(150 * attempts * attempts);
                tracing::warn!(?err, attempts, "retrying request");
                sleep(backoff).await;
            }
            Err(err) => return Err(err),
        }
    }
}

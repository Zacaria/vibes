use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

use crate::domain::{AudienceScope, FeedFilter, Post, Session, SessionTokens};

use super::client::SupabaseConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: AuthUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PostgrestPostRequest<'a> {
    author: &'a str,
    body: &'a str,
    audience: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeedRow {
    id: String,
    author: String,
    body: String,
    audience: String,
    created_at: String,
    author_handle: Option<String>,
    liked: Option<bool>,
    like_count: Option<i64>,
}

pub async fn login_with_email(
    client: &Client,
    cfg: &SupabaseConfig,
    email: &str,
    password: &str,
) -> Result<Session> {
    #[derive(Serialize)]
    struct Request<'a> {
        email: &'a str,
        password: &'a str,
    }

    let url = format!("{}/auth/v1/token?grant_type=password", cfg.url);
    let resp = client
        .post(url)
        .header("apikey", &cfg.anon_key)
        .json(&Request { email, password })
        .send()
        .await?;

    if resp.status() == StatusCode::UNAUTHORIZED {
        return Err(anyhow!("invalid credentials"));
    }

    let body: AuthResponse = resp.json().await.context("parsing auth response")?;
    let expires_at = OffsetDateTime::now_utc() + Duration::seconds(body.expires_in);
    let email = body.user.email.clone().unwrap_or_else(|| email.to_string());
    Ok(Session {
        user_id: body.user.id,
        email,
        tokens: SessionTokens {
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            expires_at,
        },
    })
}

pub async fn refresh_session(
    client: &Client,
    cfg: &SupabaseConfig,
    refresh_token: &str,
) -> Result<SessionTokens> {
    #[derive(Serialize)]
    struct Request<'a> {
        refresh_token: &'a str,
    }

    let url = format!("{}/auth/v1/token?grant_type=refresh_token", cfg.url);
    let resp = client
        .post(url)
        .header("apikey", &cfg.anon_key)
        .json(&Request { refresh_token })
        .send()
        .await?;

    if resp.status() != StatusCode::OK {
        return Err(anyhow!("refresh failed"));
    }

    let body: AuthResponse = resp.json().await?;
    Ok(SessionTokens {
        access_token: body.access_token,
        refresh_token: body.refresh_token,
        expires_at: OffsetDateTime::now_utc() + Duration::seconds(body.expires_in),
    })
}

pub async fn post_message(
    client: &Client,
    cfg: &SupabaseConfig,
    session: &Session,
    text: &str,
    scope: AudienceScope,
) -> Result<Post> {
    let url = format!("{}/rest/v1/posts", cfg.url);
    let req = PostgrestPostRequest {
        author: &session.user_id,
        body: text,
        audience: &scope.to_string(),
    };

    let resp = client
        .post(url)
        .header("apikey", &cfg.anon_key)
        .header(
            "Authorization",
            format!("Bearer {}", mask(&session.tokens.access_token)),
        )
        .json(&req)
        .send()
        .await?;

    if resp.status() != StatusCode::CREATED {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("failed to post message: {}", text));
    }

    let rows: Vec<FeedRow> = resp.json().await?;
    rows.into_iter()
        .next()
        .map(|row| row.into_post())
        .ok_or_else(|| anyhow!("empty response"))
}

pub async fn fetch_feed(
    client: &Client,
    cfg: &SupabaseConfig,
    session: &Session,
    filter: FeedFilter,
) -> Result<Vec<Post>> {
    let url = match filter {
        FeedFilter::Global => format!("{}/rest/v1/rpc/feed_global", cfg.url),
        FeedFilter::Following => format!("{}/rest/v1/rpc/feed_following", cfg.url),
        FeedFilter::Me => format!("{}/rest/v1/rpc/feed_me", cfg.url),
    };
    let resp = client
        .post(url)
        .header("apikey", &cfg.anon_key)
        .header(
            "Authorization",
            format!("Bearer {}", mask(&session.tokens.access_token)),
        )
        .json(&serde_json::json!({ "uid": session.user_id }))
        .send()
        .await?;

    if resp.status() == StatusCode::UNAUTHORIZED {
        return Err(anyhow!("session expired"));
    }

    if resp.status() != StatusCode::OK {
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("feed fetch failed: {}", text));
    }
    let rows: Vec<FeedRow> = resp.json().await?;
    Ok(rows.into_iter().map(|row| row.into_post()).collect())
}

impl FeedRow {
    fn into_post(self) -> Post {
        Post {
            id: self.id.parse().unwrap_or_default(),
            author: self.author.parse().unwrap_or_default(),
            body: self.body,
            audience: self
                .audience
                .parse()
                .unwrap_or(crate::domain::AudienceScope::Public),
            created_at: OffsetDateTime::parse(&self.created_at, &Rfc3339)
                .unwrap_or_else(|_| OffsetDateTime::now_utc()),
            author_handle: self.author_handle,
            liked: self.liked.unwrap_or(false),
            like_count: self.like_count.unwrap_or(0),
        }
    }
}

fn mask(token: &str) -> String {
    if token.len() <= 12 {
        return token.to_string();
    }
    format!("{}...{}", &token[..4], &token[token.len() - 4..])
}

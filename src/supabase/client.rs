use std::{collections::HashMap, time::Duration};

use anyhow::{anyhow, Context, Result};
use reqwest::{header, Client};
use serde::Deserialize;
use serde_json::json;
use time::OffsetDateTime;
use tokio::time::sleep;
use uuid::Uuid;

use crate::domain::{AudienceScope, FeedFilter, Post, Profile, Session};

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

#[derive(Clone)]
pub struct SupabaseClient {
    http: Client,
    config: SupabaseConfig,
}

impl SupabaseClient {
    pub fn new(config: SupabaseConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("cli-twitter/0.1")
            .build()
            .context("build reqwest client")?;
        Ok(Self { http, config })
    }

    fn base_url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn default_headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "apikey",
            header::HeaderValue::from_str(&self.config.anon_key).unwrap(),
        );
        headers
    }

    fn auth_headers(&self, session: &Session) -> Result<header::HeaderMap> {
        if let Some(token) = &session.access_token {
            let mut headers = self.default_headers();
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {}", token))
                    .context("invalid bearer token header")?,
            );
            Ok(headers)
        } else {
            Err(anyhow!("missing access token"))
        }
    }

    pub async fn login_email_password(&self, email: &str, password: &str) -> Result<Session> {
        let url = self.base_url("auth/v1/token?grant_type=password");
        let response = self
            .http
            .post(url)
            .headers(self.default_headers())
            .json(&json!({ "email": email, "password": password }))
            .send()
            .await
            .context("request password login")?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("login failed: {}", text));
        }
        let body: GoTrueSession = response.json().await.context("decode login response")?;
        Ok(body.into_session())
    }

    pub async fn refresh_session(&self, refresh: &str) -> Result<Session> {
        let url = self.base_url("auth/v1/token?grant_type=refresh_token");
        let response = self
            .http
            .post(url)
            .headers(self.default_headers())
            .json(&json!({ "refresh_token": refresh }))
            .send()
            .await
            .context("request refresh token")?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("refresh failed: {}", text));
        }
        let body: GoTrueSession = response.json().await.context("decode refresh response")?;
        Ok(body.into_session())
    }

    pub async fn current_user(&self, session: &Session) -> Result<Profile> {
        let url = self.base_url("auth/v1/user");
        let response = self
            .http
            .get(url)
            .headers(self.auth_headers(session)?)
            .send()
            .await
            .context("request current user")?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("whoami failed: {}", text));
        }
        let body: GoTrueUserResponse = response.json().await?;
        Ok(body.into_profile())
    }

    pub async fn fetch_feed(&self, filter: FeedFilter, session: &Session) -> Result<Vec<Post>> {
        let view = match filter {
            FeedFilter::Global => "rest/v1/v_feed_public",
            _ => "rest/v1/v_feed_user",
        };
        let mut url = self.base_url(view);
        match filter {
            FeedFilter::Global => {
                url.push_str("?select=*&order=created_at.desc&limit=50");
            }
            _ => {
                let user = session
                    .user_id
                    .ok_or_else(|| anyhow!("session missing user id for feed"))?;
                url.push_str(&format!(
                    "?select=*&uid=eq.{}&order=created_at.desc&limit=50",
                    user
                ));
            }
        }
        let response = self
            .http
            .get(url)
            .headers(self.auth_headers(session)?)
            .send()
            .await
            .context("request feed")?;
        if response.status().as_u16() == 404 {
            tracing::warn!("feed view missing, returning cache only");
            return Ok(vec![]);
        }
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("feed failed: {}", text));
        }
        let rows: Vec<PostRow> = response.json().await.context("decode feed response")?;
        Ok(rows.into_iter().map(PostRow::into_post).collect())
    }

    pub async fn create_post(
        &self,
        session: &Session,
        body: &str,
        scope: AudienceScope,
    ) -> Result<Post> {
        let url = self.base_url("rest/v1/posts");
        let response = self
            .http
            .post(url)
            .headers(self.auth_headers(session)?)
            .header("Prefer", "return=representation")
            .json(&json!({
                "body": body,
                "audience": scope.as_str(),
            }))
            .send()
            .await
            .context("create post request")?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("post failed: {}", text));
        }
        let mut posts: Vec<PostRow> = response.json().await.context("decode post response")?;
        posts
            .pop()
            .map(PostRow::into_post)
            .ok_or_else(|| anyhow!("post response empty"))
    }

    pub async fn follow(&self, session: &Session, handle: &str) -> Result<()> {
        let profile = self.lookup_profile_by_handle(session, handle).await?;
        let url = self.base_url("rest/v1/follows");
        let response = self
            .http
            .post(url)
            .headers(self.auth_headers(session)?)
            .header("Prefer", "resolution=merge-duplicates")
            .json(&json!({ "followee": profile.id }))
            .send()
            .await?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("follow failed: {}", text));
        }
        Ok(())
    }

    pub async fn like(&self, session: &Session, post_id: &Uuid) -> Result<()> {
        let url = self.base_url("rest/v1/likes");
        let response = self
            .http
            .post(url)
            .headers(self.auth_headers(session)?)
            .header("Prefer", "resolution=merge-duplicates")
            .json(&json!({ "post_id": post_id }))
            .send()
            .await?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("like failed: {}", text));
        }
        Ok(())
    }

    pub async fn logout(&self, session: &Session) -> Result<()> {
        let url = self.base_url("auth/v1/logout");
        let response = self
            .http
            .post(url)
            .headers(self.auth_headers(session)?)
            .send()
            .await?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            tracing::warn!("logout failed: {}", text);
        }
        Ok(())
    }

    pub async fn lookup_profile_by_handle(
        &self,
        session: &Session,
        handle: &str,
    ) -> Result<Profile> {
        let url = self.base_url("rest/v1/profiles");
        let query: Vec<(&str, String)> = vec![
            ("handle", format!("eq.{}", handle.trim_start_matches('@'))),
            ("select", "id,handle,display_name".into()),
            ("limit", "1".into()),
        ];
        let response = self
            .http
            .get(url)
            .headers(self.auth_headers(session)?)
            .query(&query)
            .send()
            .await?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("lookup profile failed: {}", text));
        }
        let mut rows: Vec<ProfileRow> = response.json().await?;
        rows.pop()
            .map(ProfileRow::into_profile)
            .ok_or_else(|| anyhow!("profile not found"))
    }

    pub async fn retry_with_backoff<F, Fut, T>(&self, mut op: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        loop {
            match op().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempt += 1;
                    if attempt > 3 {
                        return Err(err);
                    }
                    let delay = Duration::from_millis(300 * attempt as u64);
                    tracing::warn!(attempt, error = %err, "supabase op failed; retrying");
                    sleep(delay).await;
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct GoTrueSession {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    #[serde(default)]
    user: Option<GoTrueUser>,
}

impl GoTrueSession {
    fn into_session(self) -> Session {
        let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(self.expires_in);
        let user_id = self.user.as_ref().and_then(|u| u.id);
        Session {
            access_token: Some(self.access_token),
            refresh_token: Some(self.refresh_token),
            expires_at: Some(expires_at),
            user_id,
            email: self.user.and_then(|u| u.email),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GoTrueUser {
    #[serde(deserialize_with = "deserialize_uuid_opt")]
    id: Option<Uuid>,
    email: Option<String>,
}

fn deserialize_uuid_opt<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    if let Some(val) = value {
        val.parse().map(Some).map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoTrueUserResponse {
    #[serde(deserialize_with = "deserialize_uuid_opt")]
    id: Option<Uuid>,
    email: Option<String>,
    #[serde(default)]
    app_metadata: HashMap<String, serde_json::Value>,
    #[serde(default)]
    user_metadata: HashMap<String, serde_json::Value>,
}

impl GoTrueUserResponse {
    fn into_profile(self) -> Profile {
        Profile {
            id: self.id.unwrap_or_else(Uuid::nil),
            handle: self
                .user_metadata
                .get("handle")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            display_name: self
                .user_metadata
                .get("display_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PostRow {
    #[serde(deserialize_with = "deserialize_uuid")]
    id: Uuid,
    #[serde(deserialize_with = "deserialize_uuid")]
    author: Uuid,
    body: String,
    audience: String,
    created_at: String,
    #[serde(default)]
    author_handle: Option<String>,
    #[serde(default)]
    like_count: Option<i64>,
}

impl PostRow {
    fn into_post(self) -> Post {
        Post {
            id: self.id,
            author: self.author,
            body: self.body,
            audience: self.audience.parse().unwrap_or(AudienceScope::Public),
            created_at: self.created_at,
            author_handle: self.author_handle,
            like_count: self.like_count,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProfileRow {
    #[serde(deserialize_with = "deserialize_uuid")]
    id: Uuid,
    handle: String,
    #[serde(default)]
    display_name: Option<String>,
}

impl ProfileRow {
    fn into_profile(self) -> Profile {
        Profile {
            id: self.id,
            handle: self.handle,
            display_name: self.display_name,
            created_at: None,
        }
    }
}

fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}

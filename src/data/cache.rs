use anyhow::Result;
use rusqlite::params;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::domain::{AudienceScope, Post, Profile};

use super::AppDatabase;

pub struct CacheDao<'a> {
    db: &'a AppDatabase,
}

impl<'a> CacheDao<'a> {
    pub fn new(db: &'a AppDatabase) -> Self {
        Self { db }
    }

    pub fn upsert_profile(&self, profile: &Profile) -> Result<()> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO cache_profiles(id, handle, display_name, created_at) VALUES(?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET handle=excluded.handle, display_name=excluded.display_name, created_at=excluded.created_at",
            params![
                profile.id.to_string(),
                &profile.handle,
                profile.display_name.as_deref(),
                profile
                    .created_at
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| profile.created_at.to_string())
            ],
        )?;
        Ok(())
    }

    pub fn upsert_post(&self, post: &Post) -> Result<()> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO cache_posts(id, author, body, audience, created_at, author_handle, liked, like_count)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET body=excluded.body, audience=excluded.audience, created_at=excluded.created_at, author_handle=excluded.author_handle, liked=excluded.liked, like_count=excluded.like_count",
            params![
                post.id.to_string(),
                post.author.to_string(),
                &post.body,
                post.audience.to_string(),
                post
                    .created_at
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| post.created_at.to_string()),
                post.author_handle.as_deref(),
                post.liked as i64,
                post.like_count
            ],
        )?;
        Ok(())
    }

    pub fn list_posts(&self, limit: usize) -> Result<Vec<Post>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, author, body, audience, created_at, author_handle, liked, like_count FROM cache_posts ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let audience: String = row.get(3)?;
            let created_at = row
                .get::<_, String>(4)
                .ok()
                .and_then(|s| OffsetDateTime::parse(&s, &Rfc3339).ok())
                .unwrap_or_else(|| OffsetDateTime::now_utc());
            Ok(Post {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str())
                    .unwrap_or_else(|_| Uuid::nil()),
                author: Uuid::parse_str(row.get::<_, String>(1)?.as_str())
                    .unwrap_or_else(|_| Uuid::nil()),
                body: row.get(2)?,
                audience: audience.parse().unwrap_or(AudienceScope::Public),
                created_at,
                author_handle: row.get(5).ok(),
                liked: row.get::<_, i64>(6)? != 0,
                like_count: row.get(7)?,
            })
        })?;
        let posts = rows.filter_map(Result::ok).collect();
        Ok(posts)
    }
}

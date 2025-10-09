use anyhow::Result;
use rusqlite::{params, Connection, Error as SqlError, Result as SqlResult};
use uuid::Uuid;

use crate::domain::{AudienceScope, Post, Profile};

pub struct CacheDao<'a> {
    conn: &'a Connection,
}

impl<'a> CacheDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn cache_profiles(&self, profiles: &[Profile]) -> Result<()> {
        for profile in profiles {
            self.conn.execute(
                "INSERT INTO cache_profiles(id, handle, display_name, created_at) VALUES(?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET handle=excluded.handle, display_name=excluded.display_name, created_at=excluded.created_at",
                params![
                    profile.id.to_string(),
                    &profile.handle,
                    &profile.display_name,
                    &profile.created_at,
                ],
            )?;
        }
        Ok(())
    }

    pub fn cache_posts(&self, posts: &[Post]) -> Result<()> {
        for post in posts {
            self.conn.execute(
                "INSERT INTO cache_posts(id, author, body, audience, created_at, author_handle, like_count) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET body=excluded.body, audience=excluded.audience, created_at=excluded.created_at, author_handle=excluded.author_handle, like_count=excluded.like_count",
                params![
                    post.id.to_string(),
                    post.author.to_string(),
                    &post.body,
                    post.audience.as_str(),
                    &post.created_at,
                    &post.author_handle,
                    &post.like_count,
                ],
            )?;
        }
        Ok(())
    }

    pub fn list_cached_posts(&self, limit: usize) -> Result<Vec<Post>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, author, body, audience, created_at, author_handle, like_count FROM cache_posts ORDER BY datetime(created_at) DESC LIMIT ?1",
        )?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                let audience: String = row.get(3)?;
                Ok(Post {
                    id: parse_uuid(row.get(0)?)?,
                    author: parse_uuid(row.get(1)?)?,
                    body: row.get(2)?,
                    audience: audience.parse().unwrap_or(AudienceScope::Public),
                    created_at: row.get(4)?,
                    author_handle: row.get(5)?,
                    like_count: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn counts(&self) -> Result<CacheCounts> {
        let profiles: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM cache_profiles", [], |row| row.get(0))?;
        let posts: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM cache_posts", [], |row| row.get(0))?;
        let follows: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM cache_follows", [], |row| row.get(0))?;
        let likes: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM cache_likes", [], |row| row.get(0))?;
        Ok(CacheCounts {
            profiles,
            posts,
            follows,
            likes,
        })
    }
}

fn parse_uuid(value: String) -> SqlResult<Uuid> {
    Uuid::parse_str(&value).map_err(|err| {
        SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })
}

#[derive(Debug, Clone, Copy)]
pub struct CacheCounts {
    pub profiles: i64,
    pub posts: i64,
    pub follows: i64,
    pub likes: i64,
}

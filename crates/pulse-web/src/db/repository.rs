use sqlx::PgPool;
use uuid::Uuid;

use super::models::{Addon, Favorite, Playlist, PublicUser, Session, User, WatchHistory};

pub struct Repository {
    pool: PgPool,
}

impl Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // ── Users ──────────────────────────────────────────────

    pub async fn create_user(
        &self,
        email: &str,
        username: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"INSERT INTO users (id, email, username, password_hash, role, created_at, updated_at)
               VALUES ($1, $2, $3, $4, 'user', NOW(), NOW())
               RETURNING id, email, username, password_hash, avatar, role, created_at, updated_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(username)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, username, password_hash, avatar, role, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, username, password_hash, avatar, role, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn count_users(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn count_sessions(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE expires_at > NOW()")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn count_favorites(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM favorites")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn list_users(&self) -> Result<Vec<PublicUser>, sqlx::Error> {
        let users: Vec<User> = sqlx::query_as::<_, User>(
            "SELECT id, email, username, password_hash, avatar, role, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users.into_iter().map(PublicUser::from).collect())
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM favorites WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM watch_history WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ── Sessions ───────────────────────────────────────────

    pub async fn create_session(&self, user_id: Uuid, token: &str) -> Result<Session, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            r#"INSERT INTO sessions (id, user_id, token, expires_at, created_at)
               VALUES ($1, $2, $3, NOW() + INTERVAL '30 days', NOW())
               RETURNING id, user_id, token, expires_at, created_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(token)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_session_by_token(&self, token: &str) -> Result<Option<Session>, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            "SELECT id, user_id, token, expires_at, created_at FROM sessions WHERE token = $1 AND expires_at > NOW()",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_session(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sessions WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_all_sessions_for_user(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    // ── Favorites ──────────────────────────────────────────

    pub async fn get_favorites(&self, user_id: Uuid) -> Result<Vec<Favorite>, sqlx::Error> {
        sqlx::query_as::<_, Favorite>(
            "SELECT id, user_id, content_id, provider, media_type, title, poster_url, created_at FROM favorites WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_favorite(
        &self,
        user_id: Uuid,
        content_id: &str,
        provider: &str,
        media_type: &str,
        title: Option<&str>,
        poster_url: Option<&str>,
    ) -> Result<Favorite, sqlx::Error> {
        sqlx::query_as::<_, Favorite>(
            r#"INSERT INTO favorites (id, user_id, content_id, provider, media_type, title, poster_url, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
               ON CONFLICT (user_id, content_id, provider) DO NOTHING
               RETURNING id, user_id, content_id, provider, media_type, title, poster_url, created_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(content_id)
        .bind(provider)
        .bind(media_type)
        .bind(title)
        .bind(poster_url)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn remove_favorite(
        &self,
        user_id: Uuid,
        favorite_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM favorites WHERE id = $1 AND user_id = $2")
            .bind(favorite_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_favorite(
        &self,
        user_id: Uuid,
        content_id: &str,
        provider: &str,
    ) -> Result<bool, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1 AND content_id = $2 AND provider = $3",
        )
        .bind(user_id)
        .bind(content_id)
        .bind(provider)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0 > 0)
    }

    // ── Watch History ──────────────────────────────────────

    pub async fn get_watch_history(&self, user_id: Uuid) -> Result<Vec<WatchHistory>, sqlx::Error> {
        sqlx::query_as::<_, WatchHistory>(
            "SELECT id, user_id, content_id, provider, media_type, title, poster_url, season, episode, position, duration, updated_at FROM watch_history WHERE user_id = $1 ORDER BY updated_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_watch_progress(
        &self,
        user_id: Uuid,
        content_id: &str,
        provider: &str,
        media_type: &str,
        title: Option<&str>,
        poster_url: Option<&str>,
        season: Option<i32>,
        episode: Option<i32>,
        position: f64,
        duration: f64,
    ) -> Result<WatchHistory, sqlx::Error> {
        sqlx::query_as::<_, WatchHistory>(
            r#"INSERT INTO watch_history (id, user_id, content_id, provider, media_type, title, poster_url, season, episode, position, duration, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
               ON CONFLICT (user_id, content_id, provider) DO UPDATE SET
                 season = EXCLUDED.season,
                 episode = EXCLUDED.episode,
                 position = EXCLUDED.position,
                 duration = EXCLUDED.duration,
                 title = COALESCE(EXCLUDED.title, watch_history.title),
                 poster_url = COALESCE(EXCLUDED.poster_url, watch_history.poster_url),
                 updated_at = NOW()
               RETURNING id, user_id, content_id, provider, media_type, title, poster_url, season, episode, position, duration, updated_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(content_id)
        .bind(provider)
        .bind(media_type)
        .bind(title)
        .bind(poster_url)
        .bind(season)
        .bind(episode)
        .bind(position)
        .bind(duration)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn remove_watch_history(
        &self,
        user_id: Uuid,
        history_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM watch_history WHERE id = $1 AND user_id = $2")
            .bind(history_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_continue_watching(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<WatchHistory>, sqlx::Error> {
        sqlx::query_as::<_, WatchHistory>(
            r#"SELECT DISTINCT ON (content_id, provider)
               id, user_id, content_id, provider, media_type, title, poster_url, season, episode, position, duration, updated_at
               FROM watch_history
               WHERE user_id = $1 AND duration > 0 AND position > 0 AND position < duration * 0.95
               ORDER BY content_id, provider, updated_at DESC
               LIMIT $2"#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // ── Playlists ──────────────────────────────────────────

    pub async fn get_playlists(&self, user_id: Uuid) -> Result<Vec<Playlist>, sqlx::Error> {
        sqlx::query_as::<_, Playlist>(
            "SELECT id, user_id, name, url, enabled, channel_count, last_refreshed, created_at FROM playlists WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_playlist(
        &self,
        user_id: Uuid,
        name: &str,
        url: &str,
    ) -> Result<Playlist, sqlx::Error> {
        sqlx::query_as::<_, Playlist>(
            r#"INSERT INTO playlists (id, user_id, name, url, enabled, created_at)
               VALUES ($1, $2, $3, $4, true, NOW())
               RETURNING id, user_id, name, url, enabled, channel_count, last_refreshed, created_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(name)
        .bind(url)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn remove_playlist(
        &self,
        user_id: Uuid,
        playlist_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM playlists WHERE id = $1 AND user_id = $2")
            .bind(playlist_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn toggle_playlist(
        &self,
        user_id: Uuid,
        playlist_id: Uuid,
    ) -> Result<Playlist, sqlx::Error> {
        sqlx::query_as::<_, Playlist>(
            r#"UPDATE playlists SET enabled = NOT enabled WHERE id = $1 AND user_id = $2
               RETURNING id, user_id, name, url, enabled, channel_count, last_refreshed, created_at"#,
        )
        .bind(playlist_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }

    // ── Addons ─────────────────────────────────────────────

    pub async fn get_addons(&self, user_id: Uuid) -> Result<Vec<Addon>, sqlx::Error> {
        sqlx::query_as::<_, Addon>(
            "SELECT id, user_id, name, manifest_url, enabled, created_at FROM addons WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_addon(
        &self,
        user_id: Uuid,
        name: &str,
        manifest_url: &str,
    ) -> Result<Addon, sqlx::Error> {
        sqlx::query_as::<_, Addon>(
            r#"INSERT INTO addons (id, user_id, name, manifest_url, enabled, created_at)
               VALUES ($1, $2, $3, $4, true, NOW())
               RETURNING id, user_id, name, manifest_url, enabled, created_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(name)
        .bind(manifest_url)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn remove_addon(&self, user_id: Uuid, addon_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM addons WHERE id = $1 AND user_id = $2")
            .bind(addon_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn toggle_addon(&self, user_id: Uuid, addon_id: Uuid) -> Result<Addon, sqlx::Error> {
        sqlx::query_as::<_, Addon>(
            r#"UPDATE addons SET enabled = NOT enabled WHERE id = $1 AND user_id = $2
               RETURNING id, user_id, name, manifest_url, enabled, created_at"#,
        )
        .bind(addon_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }
}

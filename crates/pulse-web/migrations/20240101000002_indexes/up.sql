-- Performance indexes for common query patterns

-- Users
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users (role);

-- Sessions
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions (token);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions (expires_at);

-- Favorites
CREATE INDEX IF NOT EXISTS idx_favorites_user_id ON favorites (user_id);
CREATE INDEX IF NOT EXISTS idx_favorites_user_content ON favorites (user_id, content_id, provider);

-- Watch History
CREATE INDEX IF NOT EXISTS idx_watch_history_user_id ON watch_history (user_id);
CREATE INDEX IF NOT EXISTS idx_watch_history_user_content ON watch_history (user_id, content_id);
CREATE INDEX IF NOT EXISTS idx_watch_history_updated_at ON watch_history (updated_at DESC);

-- Playlists
CREATE INDEX IF NOT EXISTS idx_playlists_user_id ON playlists (user_id);

-- Addons
CREATE INDEX IF NOT EXISTS idx_addons_user_id ON addons (user_id);

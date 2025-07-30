-- Create oauth_providers table
CREATE TABLE IF NOT EXISTS oauth_providers (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    expires_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(provider, provider_user_id)
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_oauth_providers_user_id ON oauth_providers(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_providers_provider ON oauth_providers(provider);
CREATE INDEX IF NOT EXISTS idx_oauth_providers_provider_user_id ON oauth_providers(provider_user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_providers_created_at ON oauth_providers(created_at);
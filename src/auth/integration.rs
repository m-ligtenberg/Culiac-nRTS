use crate::auth::{start_auth_server_background, AuthError};
use std::env;

pub struct GameAuthIntegration {
    pub auth_server_port: u16,
    pub auth_enabled: bool,
}

impl Default for GameAuthIntegration {
    fn default() -> Self {
        Self {
            auth_server_port: 8080,
            auth_enabled: true,
        }
    }
}

impl GameAuthIntegration {
    pub fn new() -> Self {
        let auth_server_port = env::var("AUTH_SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        let auth_enabled = env::var("AUTH_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Self {
            auth_server_port,
            auth_enabled,
        }
    }

    pub async fn initialize(&self) -> Result<(), AuthError> {
        if !self.auth_enabled {
            println!("Authentication system disabled");
            return Ok(());
        }

        println!("Initializing authentication system...");

        // Set up environment variables if not set
        self.setup_default_environment();

        // Start authentication server in background
        start_auth_server_background(self.auth_server_port).await?;

        println!("Authentication system initialized successfully");
        println!("API endpoints available at: http://localhost:{}/api", self.auth_server_port);
        println!("");
        println!("Available endpoints:");
        println!("  POST /api/auth/register          - User registration");
        println!("  POST /api/auth/login             - User login");
        println!("  POST /api/auth/logout            - User logout");
        println!("  POST /api/auth/refresh           - Refresh access token");
        println!("  GET  /api/auth/profile           - Get user profile");
        println!("  PUT  /api/auth/profile           - Update user profile");
        println!("  POST /api/auth/change-password   - Change password");
        println!("  POST /api/auth/reset-password    - Request password reset");
        println!("  GET  /api/auth/oauth/google      - Google OAuth login");
        println!("  GET  /api/auth/oauth/github      - GitHub OAuth login");
        println!("  GET  /api/auth/oauth/discord     - Discord OAuth login");
        println!("  GET  /api/info/health            - Health check");
        println!("  GET  /api/info/stats             - Service statistics");
        println!("");

        Ok(())
    }

    fn setup_default_environment(&self) {
        // Set up default JWT secret if not provided
        if env::var("JWT_SECRET").is_err() {
            env::set_var("JWT_SECRET", "culiacan-rts-development-secret-change-in-production");
            println!("⚠️  Using default JWT secret. Set JWT_SECRET environment variable in production!");
        }

        // Set up default database URL if not provided
        if env::var("DATABASE_URL").is_err() {
            env::set_var("DATABASE_URL", "sqlite:culiacan_rts_auth.db");
            println!("Using SQLite database: culiacan_rts_auth.db");
        }

        // Set up default admin user if environment variables are provided
        if let (Ok(_), Ok(_), Ok(_)) = (
            env::var("ADMIN_USERNAME"),
            env::var("ADMIN_EMAIL"),
            env::var("ADMIN_PASSWORD"),
        ) {
            println!("Admin user will be created from environment variables");
        } else {
            // Set up default admin for development
            env::set_var("ADMIN_USERNAME", "admin");
            env::set_var("ADMIN_EMAIL", "admin@culiacan-rts.local");
            env::set_var("ADMIN_PASSWORD", "admin123");
            println!("⚠️  Creating default admin user (admin/admin123). Change in production!");
        }

        // OAuth configuration hints
        if env::var("GOOGLE_CLIENT_ID").is_err() {
            println!("💡 Set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET for Google OAuth");
        }
        if env::var("GITHUB_CLIENT_ID").is_err() {
            println!("💡 Set GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET for GitHub OAuth");
        }
        if env::var("DISCORD_CLIENT_ID").is_err() {
            println!("💡 Set DISCORD_CLIENT_ID and DISCORD_CLIENT_SECRET for Discord OAuth");
        }
    }

    pub fn get_api_base_url(&self) -> String {
        format!("http://localhost:{}/api", self.auth_server_port)
    }

    pub fn is_enabled(&self) -> bool {
        self.auth_enabled
    }
}

// Helper function to easily integrate auth into the game
pub async fn setup_game_authentication() -> Result<GameAuthIntegration, AuthError> {
    let integration = GameAuthIntegration::new();
    integration.initialize().await?;
    Ok(integration)
}

// Example usage in main.rs:
/*
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize authentication system
    let _auth_integration = crate::auth::setup_game_authentication().await?;
    
    // Start the Bevy game
    App::new()
        // ... rest of your Bevy setup
        .run();
    
    Ok(())
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_integration() {
        let integration = GameAuthIntegration::default();
        assert_eq!(integration.auth_server_port, 8080);
        assert!(integration.auth_enabled);
    }

    #[test]
    fn test_integration_from_env() {
        env::set_var("AUTH_SERVER_PORT", "9090");
        env::set_var("AUTH_ENABLED", "false");
        
        let integration = GameAuthIntegration::new();
        assert_eq!(integration.auth_server_port, 9090);
        assert!(!integration.auth_enabled);
        
        // Clean up
        env::remove_var("AUTH_SERVER_PORT");
        env::remove_var("AUTH_ENABLED");
    }

    #[test]
    fn test_api_base_url() {
        let integration = GameAuthIntegration {
            auth_server_port: 3000,
            auth_enabled: true,
        };
        
        assert_eq!(integration.get_api_base_url(), "http://localhost:3000/api");
    }
}
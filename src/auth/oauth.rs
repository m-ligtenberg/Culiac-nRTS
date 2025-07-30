use crate::auth::{AuthError, CreateUserRequest, User, UserRole};
use oauth2::url::Url;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub google: Option<GoogleConfig>,
    pub github: Option<GitHubConfig>,
    pub discord: Option<DiscordConfig>,
}

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client: BasicClient,
    pub redirect_url: String,
}

#[derive(Debug, Clone)]
pub struct GitHubConfig {
    pub client: BasicClient,
    pub redirect_url: String,
}

#[derive(Debug, Clone)]
pub struct DiscordConfig {
    pub client: BasicClient,
    pub redirect_url: String,
}

impl OAuthConfig {
    pub fn new() -> Result<Self, AuthError> {
        let google = Self::setup_google()?;
        let github = Self::setup_github()?;
        let discord = Self::setup_discord()?;

        Ok(Self {
            google,
            github,
            discord,
        })
    }

    fn setup_google() -> Result<Option<GoogleConfig>, AuthError> {
        let client_id = env::var("GOOGLE_CLIENT_ID").ok();
        let client_secret = env::var("GOOGLE_CLIENT_SECRET").ok();
        let redirect_url = env::var("GOOGLE_REDIRECT_URL").ok();

        if let (Some(client_id), Some(client_secret), Some(redirect_url)) =
            (client_id, client_secret, redirect_url)
        {
            let client = BasicClient::new(
                ClientId::new(client_id),
                Some(ClientSecret::new(client_secret)),
                AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())
                    .map_err(|_| AuthError::OAuthError("Invalid Google auth URL".to_string()))?,
                Some(
                    TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).map_err(
                        |_| AuthError::OAuthError("Invalid Google token URL".to_string()),
                    )?,
                ),
            )
            .set_redirect_uri(
                RedirectUrl::new(redirect_url.clone()).map_err(|_| {
                    AuthError::OAuthError("Invalid Google redirect URL".to_string())
                })?,
            );

            Ok(Some(GoogleConfig {
                client,
                redirect_url,
            }))
        } else {
            Ok(None)
        }
    }

    fn setup_github() -> Result<Option<GitHubConfig>, AuthError> {
        let client_id = env::var("GITHUB_CLIENT_ID").ok();
        let client_secret = env::var("GITHUB_CLIENT_SECRET").ok();
        let redirect_url = env::var("GITHUB_REDIRECT_URL").ok();

        if let (Some(client_id), Some(client_secret), Some(redirect_url)) =
            (client_id, client_secret, redirect_url)
        {
            let client = BasicClient::new(
                ClientId::new(client_id),
                Some(ClientSecret::new(client_secret)),
                AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
                    .map_err(|_| AuthError::OAuthError("Invalid GitHub auth URL".to_string()))?,
                Some(
                    TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                        .map_err(|_| {
                            AuthError::OAuthError("Invalid GitHub token URL".to_string())
                        })?,
                ),
            )
            .set_redirect_uri(
                RedirectUrl::new(redirect_url.clone()).map_err(|_| {
                    AuthError::OAuthError("Invalid GitHub redirect URL".to_string())
                })?,
            );

            Ok(Some(GitHubConfig {
                client,
                redirect_url,
            }))
        } else {
            Ok(None)
        }
    }

    fn setup_discord() -> Result<Option<DiscordConfig>, AuthError> {
        let client_id = env::var("DISCORD_CLIENT_ID").ok();
        let client_secret = env::var("DISCORD_CLIENT_SECRET").ok();
        let redirect_url = env::var("DISCORD_REDIRECT_URL").ok();

        if let (Some(client_id), Some(client_secret), Some(redirect_url)) =
            (client_id, client_secret, redirect_url)
        {
            let client = BasicClient::new(
                ClientId::new(client_id),
                Some(ClientSecret::new(client_secret)),
                AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
                    .map_err(|_| AuthError::OAuthError("Invalid Discord auth URL".to_string()))?,
                Some(
                    TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).map_err(
                        |_| AuthError::OAuthError("Invalid Discord token URL".to_string()),
                    )?,
                ),
            )
            .set_redirect_uri(
                RedirectUrl::new(redirect_url.clone()).map_err(|_| {
                    AuthError::OAuthError("Invalid Discord redirect URL".to_string())
                })?,
            );

            Ok(Some(DiscordConfig {
                client,
                redirect_url,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OAuthAuthorizationUrl {
    pub url: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

// Google OAuth user info
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

// GitHub OAuth user info
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUserInfo {
    pub id: u64,
    pub login: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

// Discord OAuth user info
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordUserInfo {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub email: Option<String>,
    pub verified: Option<bool>,
    pub avatar: Option<String>,
}

pub struct OAuthService {
    config: OAuthConfig,
    http_client: Client,
}

impl OAuthService {
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    // Google OAuth methods
    pub fn google_auth_url(&self) -> Result<OAuthAuthorizationUrl, AuthError> {
        let google_config = self
            .config
            .google
            .as_ref()
            .ok_or_else(|| AuthError::OAuthError("Google OAuth not configured".to_string()))?;

        let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = google_config
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok(OAuthAuthorizationUrl {
            url: auth_url.to_string(),
            state: csrf_token.secret().clone(),
        })
    }

    pub async fn google_exchange_code(&self, code: &str) -> Result<GoogleUserInfo, AuthError> {
        let google_config = self
            .config
            .google
            .as_ref()
            .ok_or_else(|| AuthError::OAuthError("Google OAuth not configured".to_string()))?;

        let token_result = google_config
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::OAuthError(format!("Token exchange failed: {}", e)))?;

        let access_token = token_result.access_token().secret();

        // Get user info from Google
        let user_info = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::OAuthError(format!("Failed to get user info: {}", e)))?
            .json::<GoogleUserInfo>()
            .await
            .map_err(|e| AuthError::OAuthError(format!("Failed to parse user info: {}", e)))?;

        Ok(user_info)
    }

    // GitHub OAuth methods
    pub fn github_auth_url(&self) -> Result<OAuthAuthorizationUrl, AuthError> {
        let github_config = self
            .config
            .github
            .as_ref()
            .ok_or_else(|| AuthError::OAuthError("GitHub OAuth not configured".to_string()))?;

        let (auth_url, csrf_token) = github_config
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user:email".to_string()))
            .url();

        Ok(OAuthAuthorizationUrl {
            url: auth_url.to_string(),
            state: csrf_token.secret().clone(),
        })
    }

    pub async fn github_exchange_code(&self, code: &str) -> Result<GitHubUserInfo, AuthError> {
        let github_config = self
            .config
            .github
            .as_ref()
            .ok_or_else(|| AuthError::OAuthError("GitHub OAuth not configured".to_string()))?;

        let token_result = github_config
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::OAuthError(format!("Token exchange failed: {}", e)))?;

        let access_token = token_result.access_token().secret();

        // Get user info from GitHub
        let user_info = self
            .http_client
            .get("https://api.github.com/user")
            .bearer_auth(access_token)
            .header("User-Agent", "CuliacanRTS")
            .send()
            .await
            .map_err(|e| AuthError::OAuthError(format!("Failed to get user info: {}", e)))?
            .json::<GitHubUserInfo>()
            .await
            .map_err(|e| AuthError::OAuthError(format!("Failed to parse user info: {}", e)))?;

        Ok(user_info)
    }

    // Discord OAuth methods
    pub fn discord_auth_url(&self) -> Result<OAuthAuthorizationUrl, AuthError> {
        let discord_config = self
            .config
            .discord
            .as_ref()
            .ok_or_else(|| AuthError::OAuthError("Discord OAuth not configured".to_string()))?;

        let (auth_url, csrf_token) = discord_config
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("identify".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .url();

        Ok(OAuthAuthorizationUrl {
            url: auth_url.to_string(),
            state: csrf_token.secret().clone(),
        })
    }

    pub async fn discord_exchange_code(&self, code: &str) -> Result<DiscordUserInfo, AuthError> {
        let discord_config = self
            .config
            .discord
            .as_ref()
            .ok_or_else(|| AuthError::OAuthError("Discord OAuth not configured".to_string()))?;

        let token_result = discord_config
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::OAuthError(format!("Token exchange failed: {}", e)))?;

        let access_token = token_result.access_token().secret();

        // Get user info from Discord
        let user_info = self
            .http_client
            .get("https://discord.com/api/users/@me")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::OAuthError(format!("Failed to get user info: {}", e)))?
            .json::<DiscordUserInfo>()
            .await
            .map_err(|e| AuthError::OAuthError(format!("Failed to parse user info: {}", e)))?;

        Ok(user_info)
    }
}

// Helper functions to convert OAuth user info to CreateUserRequest
pub fn google_user_to_create_request(user_info: GoogleUserInfo) -> CreateUserRequest {
    CreateUserRequest {
        username: user_info.name.clone(),
        email: user_info.email,
        password: format!("oauth_google_{}", user_info.id), // Temp password, will be replaced
    }
}

pub fn github_user_to_create_request(
    user_info: GitHubUserInfo,
) -> Result<CreateUserRequest, AuthError> {
    let email = user_info
        .email
        .ok_or_else(|| AuthError::OAuthError("GitHub user has no public email".to_string()))?;

    Ok(CreateUserRequest {
        username: user_info.login,
        email,
        password: format!("oauth_github_{}", user_info.id), // Temp password, will be replaced
    })
}

pub fn discord_user_to_create_request(
    user_info: DiscordUserInfo,
) -> Result<CreateUserRequest, AuthError> {
    let email = user_info
        .email
        .ok_or_else(|| AuthError::OAuthError("Discord user has no email".to_string()))?;

    Ok(CreateUserRequest {
        username: format!("{}#{}", user_info.username, user_info.discriminator),
        email,
        password: format!("oauth_discord_{}", user_info.id), // Temp password, will be replaced
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_user_conversion() {
        let google_user = GoogleUserInfo {
            id: "123456789".to_string(),
            email: "test@gmail.com".to_string(),
            verified_email: true,
            name: "Test User".to_string(),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            picture: None,
        };

        let create_request = google_user_to_create_request(google_user);
        assert_eq!(create_request.username, "Test User");
        assert_eq!(create_request.email, "test@gmail.com");
        assert!(create_request.password.starts_with("oauth_google_"));
    }

    #[test]
    fn test_github_user_conversion() {
        let github_user = GitHubUserInfo {
            id: 123456789,
            login: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            name: Some("Test User".to_string()),
            avatar_url: None,
        };

        let create_request = github_user_to_create_request(github_user).unwrap();
        assert_eq!(create_request.username, "testuser");
        assert_eq!(create_request.email, "test@example.com");
        assert!(create_request.password.starts_with("oauth_github_"));
    }

    #[test]
    fn test_discord_user_conversion() {
        let discord_user = DiscordUserInfo {
            id: "123456789".to_string(),
            username: "testuser".to_string(),
            discriminator: "1234".to_string(),
            email: Some("test@example.com".to_string()),
            verified: Some(true),
            avatar: None,
        };

        let create_request = discord_user_to_create_request(discord_user).unwrap();
        assert_eq!(create_request.username, "testuser#1234");
        assert_eq!(create_request.email, "test@example.com");
        assert!(create_request.password.starts_with("oauth_discord_"));
    }
}

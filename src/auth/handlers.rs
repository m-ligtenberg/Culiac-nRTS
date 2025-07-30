use crate::auth::{
    AuthDatabase, AuthError, AuthResponse, AuthService, ChangePasswordRequest, Claims,
    ConfirmResetPasswordRequest, CreateUserRequest, CurrentUser, JwtService, LoginRequest,
    OAuthCallback, OAuthService, OptionalCurrentUser, ResetPasswordRequest, UpdateUserRequest,
    UserInfo, discord_user_to_create_request, github_user_to_create_request,
    google_user_to_create_request,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub struct AuthHandlers {
    pub auth_db: Arc<AuthDatabase>,
    pub jwt_service: Arc<JwtService>,
    pub oauth_service: Arc<OAuthService>,
}

impl AuthHandlers {
    pub fn new(
        auth_db: AuthDatabase,
        jwt_service: JwtService,
        oauth_service: OAuthService,
    ) -> Self {
        Self {
            auth_db: Arc::new(auth_db),
            jwt_service: Arc::new(jwt_service),
            oauth_service: Arc::new(oauth_service),
        }
    }
}

// Registration endpoint
pub async fn register(
    State(handlers): State<Arc<AuthHandlers>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate input
    request.validate()?;

    // Create user
    let user = handlers.auth_db.create_user(request).await?;

    // Generate tokens
    let (access_token, refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

    // Create session
    let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
    handlers.auth_db.create_session(user.id, &refresh_token, expires_at, None, None).await?;

    // Create email verification token
    let verification_token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    handlers.auth_db.create_email_verification(user.id, &verification_token).await?;

    // TODO: Send verification email
    // send_verification_email(&user.email, &verification_token).await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
        user: UserInfo::from(user),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// Login endpoint
pub async fn login(
    State(handlers): State<Arc<AuthHandlers>>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate input
    request.validate()?;

    // Find user by username or email
    let user = handlers.auth_db.get_user_by_identifier(&request.identifier).await?;

    // Check if account is active
    if !user.is_active {
        return Err(AuthError::AccountDeactivated);
    }

    // Verify password
    if !handlers.auth_db.verify_password(&user, &request.password).await? {
        return Err(AuthError::InvalidCredentials);
    }

    // Update last login
    handlers.auth_db.update_last_login(user.id).await?;

    // Generate tokens
    let (access_token, refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

    // Create session
    let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).map(String::from);
    let ip_address = headers.get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(String::from);

    handlers.auth_db.create_session(user.id, &refresh_token, expires_at, user_agent, ip_address).await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
        user: UserInfo::from(user),
    };

    Ok(Json(response))
}

// Logout endpoint
pub async fn logout(
    State(handlers): State<Arc<AuthHandlers>>,
    current_user: CurrentUser,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AuthError> {
    // Extract refresh token from request body
    let refresh_token = body.get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or(AuthError::InvalidRefreshToken)?;

    // Find and delete the session
    if let Ok(session) = handlers.auth_db.get_session_by_token(refresh_token).await {
        if session.user_id == current_user.id {
            handlers.auth_db.delete_session(session.id).await?;
        }
    }

    Ok(Json(json!({"message": "Logged out successfully"})))
}

// Refresh token endpoint
pub async fn refresh_token(
    State(handlers): State<Arc<AuthHandlers>>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AuthError> {
    let refresh_token = body.get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or(AuthError::InvalidRefreshToken)?;

    // Validate refresh token
    let claims = handlers.jwt_service.validate_token(refresh_token)?;
    
    // Get session
    let session = handlers.auth_db.get_session_by_token(refresh_token).await?;
    
    // Check if session is expired
    if session.expires_at < Utc::now() {
        handlers.auth_db.delete_session(session.id).await?;
        return Err(AuthError::TokenExpired);
    }

    // Get user
    let user = handlers.auth_db.get_user_by_id(session.user_id).await?;

    // Check if user is still active
    if !user.is_active {
        handlers.auth_db.delete_session(session.id).await?;
        return Err(AuthError::AccountDeactivated);
    }

    // Generate new tokens
    let (new_access_token, new_refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

    // Update session with new refresh token
    handlers.auth_db.delete_session(session.id).await?;
    let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
    handlers.auth_db.create_session(user.id, &new_refresh_token, expires_at, session.user_agent, session.ip_address).await?;

    let response = AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
        user: UserInfo::from(user),
    };

    Ok(Json(response))
}

// Get current user profile
pub async fn get_profile(
    State(handlers): State<Arc<AuthHandlers>>,
    current_user: CurrentUser,
) -> Result<impl IntoResponse, AuthError> {
    let user = handlers.auth_db.get_user_by_id(current_user.id).await?;
    Ok(Json(UserInfo::from(user)))
}

// Update user profile
pub async fn update_profile(
    State(handlers): State<Arc<AuthHandlers>>,
    current_user: CurrentUser,
    Json(request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate input
    request.validate()?;

    // Update username if provided
    if let Some(username) = &request.username {
        handlers.auth_db.update_user_username(current_user.id, username).await?;
    }

    // Update email if provided
    if let Some(email) = &request.email {
        handlers.auth_db.update_user_email(current_user.id, email).await?;
        
        // Create new email verification token
        let verification_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        
        handlers.auth_db.create_email_verification(current_user.id, &verification_token).await?;
        
        // TODO: Send verification email
        // send_verification_email(email, &verification_token).await?;
    }

    // Get updated user
    let user = handlers.auth_db.get_user_by_id(current_user.id).await?;
    Ok(Json(UserInfo::from(user)))
}

// Change password endpoint
pub async fn change_password(
    State(handlers): State<Arc<AuthHandlers>>,
    current_user: CurrentUser,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate input
    request.validate()?;

    // Get current user
    let user = handlers.auth_db.get_user_by_id(current_user.id).await?;

    // Verify current password
    if !handlers.auth_db.verify_password(&user, &request.current_password).await? {
        return Err(AuthError::InvalidCredentials);
    }

    // Update password
    handlers.auth_db.update_user_password(current_user.id, &request.new_password).await?;

    // Invalidate all sessions except current one (force re-login on other devices)
    handlers.auth_db.delete_user_sessions(current_user.id).await?;

    Ok(Json(json!({"message": "Password updated successfully"})))
}

// Request password reset
pub async fn request_password_reset(
    State(handlers): State<Arc<AuthHandlers>>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate input
    request.validate()?;

    // Find user by email (don't reveal if user exists or not)
    if let Ok(user) = handlers.auth_db.get_user_by_email(&request.email).await {
        // Generate reset token
        let reset_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Create password reset record
        handlers.auth_db.create_password_reset(user.id, &reset_token).await?;

        // TODO: Send password reset email
        // send_password_reset_email(&user.email, &reset_token).await?;
    }

    // Always return success to prevent user enumeration
    Ok(Json(json!({"message": "If the email exists, a password reset link has been sent"})))
}

// Confirm password reset
pub async fn confirm_password_reset(
    State(handlers): State<Arc<AuthHandlers>>,
    Json(request): Json<ConfirmResetPasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate input
    request.validate()?;

    // Find password reset record
    let reset = handlers.auth_db.get_password_reset_by_token(&request.token).await?;

    // Update user password
    handlers.auth_db.update_user_password(reset.user_id, &request.new_password).await?;

    // Mark reset token as used
    handlers.auth_db.mark_password_reset_used(reset.id).await?;

    // Invalidate all user sessions (force re-login)
    handlers.auth_db.delete_user_sessions(reset.user_id).await?;

    Ok(Json(json!({"message": "Password reset successfully"})))
}

// Verify email endpoint
pub async fn verify_email(
    State(handlers): State<Arc<AuthHandlers>>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, AuthError> {
    // Find email verification record
    let verification = handlers.auth_db.get_email_verification_by_token(&token).await?;

    // Verify user email
    handlers.auth_db.verify_user_email(verification.user_id).await?;

    // Mark verification as used
    handlers.auth_db.mark_email_verification_used(verification.id).await?;

    Ok(Json(json!({"message": "Email verified successfully"})))
}

// OAuth - Google
pub async fn google_oauth_url(
    State(handlers): State<Arc<AuthHandlers>>,
) -> Result<impl IntoResponse, AuthError> {
    let auth_url = handlers.oauth_service.google_auth_url()?;
    Ok(Json(auth_url))
}

pub async fn google_oauth_callback(
    State(handlers): State<Arc<AuthHandlers>>,
    Query(callback): Query<OAuthCallback>,
) -> Result<impl IntoResponse, AuthError> {
    // Exchange code for user info
    let google_user = handlers.oauth_service.google_exchange_code(&callback.code).await?;

    // Try to find existing OAuth provider
    if let Ok(oauth_provider) = handlers.auth_db.get_oauth_provider("google", &google_user.id).await {
        // User exists, log them in
        let user = handlers.auth_db.get_user_by_id(oauth_provider.user_id).await?;
        
        if !user.is_active {
            return Err(AuthError::AccountDeactivated);
        }

        // Update last login
        handlers.auth_db.update_last_login(user.id).await?;

        // Generate tokens
        let (access_token, refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

        // Create session
        let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
        handlers.auth_db.create_session(user.id, &refresh_token, expires_at, None, None).await?;

        let response = AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
            user: UserInfo::from(user),
        };

        return Ok(Json(response));
    }

    // User doesn't exist, create new account
    let create_request = google_user_to_create_request(google_user.clone());
    let mut user = handlers.auth_db.create_user(create_request).await?;

    // Mark as verified since Google verified the email
    if google_user.verified_email {
        handlers.auth_db.verify_user_email(user.id).await?;
        user.is_verified = true;
    }

    // Create OAuth provider record
    handlers.auth_db.create_oauth_provider(
        user.id,
        "google",
        &google_user.id,
        None, // Don't store access token for privacy
        None,
        None,
    ).await?;

    // Generate tokens
    let (access_token, refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

    // Create session
    let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
    handlers.auth_db.create_session(user.id, &refresh_token, expires_at, None, None).await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
        user: UserInfo::from(user),
    };

    Ok(Json(response))
}

// OAuth - GitHub
pub async fn github_oauth_url(
    State(handlers): State<Arc<AuthHandlers>>,
) -> Result<impl IntoResponse, AuthError> {
    let auth_url = handlers.oauth_service.github_auth_url()?;
    Ok(Json(auth_url))
}

pub async fn github_oauth_callback(
    State(handlers): State<Arc<AuthHandlers>>,
    Query(callback): Query<OAuthCallback>,
) -> Result<impl IntoResponse, AuthError> {
    // Exchange code for user info
    let github_user = handlers.oauth_service.github_exchange_code(&callback.code).await?;

    // Try to find existing OAuth provider
    if let Ok(oauth_provider) = handlers.auth_db.get_oauth_provider("github", &github_user.id.to_string()).await {
        // User exists, log them in
        let user = handlers.auth_db.get_user_by_id(oauth_provider.user_id).await?;
        
        if !user.is_active {
            return Err(AuthError::AccountDeactivated);
        }

        // Update last login
        handlers.auth_db.update_last_login(user.id).await?;

        // Generate tokens
        let (access_token, refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

        // Create session
        let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
        handlers.auth_db.create_session(user.id, &refresh_token, expires_at, None, None).await?;

        let response = AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
            user: UserInfo::from(user),
        };

        return Ok(Json(response));
    }

    // User doesn't exist, create new account
    let create_request = github_user_to_create_request(github_user.clone())?;
    let user = handlers.auth_db.create_user(create_request).await?;

    // Create OAuth provider record
    handlers.auth_db.create_oauth_provider(
        user.id,
        "github",
        &github_user.id.to_string(),
        None,
        None,
        None,
    ).await?;

    // Generate tokens
    let (access_token, refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

    // Create session
    let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
    handlers.auth_db.create_session(user.id, &refresh_token, expires_at, None, None).await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
        user: UserInfo::from(user),
    };

    Ok(Json(response))
}

// OAuth - Discord
pub async fn discord_oauth_url(
    State(handlers): State<Arc<AuthHandlers>>,
) -> Result<impl IntoResponse, AuthError> {
    let auth_url = handlers.oauth_service.discord_auth_url()?;
    Ok(Json(auth_url))
}

pub async fn discord_oauth_callback(
    State(handlers): State<Arc<AuthHandlers>>,
    Query(callback): Query<OAuthCallback>,
) -> Result<impl IntoResponse, AuthError> {
    // Exchange code for user info
    let discord_user = handlers.oauth_service.discord_exchange_code(&callback.code).await?;

    // Try to find existing OAuth provider
    if let Ok(oauth_provider) = handlers.auth_db.get_oauth_provider("discord", &discord_user.id).await {
        // User exists, log them in
        let user = handlers.auth_db.get_user_by_id(oauth_provider.user_id).await?;
        
        if !user.is_active {
            return Err(AuthError::AccountDeactivated);
        }

        // Update last login
        handlers.auth_db.update_last_login(user.id).await?;

        // Generate tokens
        let (access_token, refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

        // Create session
        let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
        handlers.auth_db.create_session(user.id, &refresh_token, expires_at, None, None).await?;

        let response = AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
            user: UserInfo::from(user),
        };

        return Ok(Json(response));
    }

    // User doesn't exist, create new account
    let create_request = discord_user_to_create_request(discord_user.clone())?;
    let mut user = handlers.auth_db.create_user(create_request).await?;

    // Mark as verified if Discord verified the email
    if discord_user.verified.unwrap_or(false) {
        handlers.auth_db.verify_user_email(user.id).await?;
        user.is_verified = true;
    }

    // Create OAuth provider record
    handlers.auth_db.create_oauth_provider(
        user.id,
        "discord",
        &discord_user.id,
        None,
        None,
        None,
    ).await?;

    // Generate tokens
    let (access_token, refresh_token) = handlers.jwt_service.generate_token_pair(&user)?;

    // Create session
    let expires_at = Utc::now() + chrono::Duration::seconds(handlers.jwt_service.get_refresh_token_expiry_seconds());
    handlers.auth_db.create_session(user.id, &refresh_token, expires_at, None, None).await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: handlers.jwt_service.get_access_token_expiry_seconds(),
        user: UserInfo::from(user),
    };

    Ok(Json(response))
}
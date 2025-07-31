use crate::auth::{
    admin_only_middleware, auth_middleware, cleanup_expired_data, create_admin_user,
    initialize_database, moderator_or_admin_middleware, optional_auth_middleware,
    rate_limit_middleware, AuthDatabase, AuthError, AuthHandlers, AuthService, CurrentUser,
    JwtService, OAuthConfig, OAuthService, OptionalCurrentUser, RateLimiter,
};
use axum::{
    extract::State,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method, StatusCode,
    },
    middleware,
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use bevy::log::{error, info};
use serde_json::json;
use std::{env, sync::Arc, time::Duration};
use tokio::time;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

pub struct AuthServer {
    pub router: Router,
    pub handlers: Arc<AuthHandlers>,
}

impl AuthServer {
    pub async fn new() -> Result<Self, AuthError> {
        // Initialize database
        let pool = initialize_database().await?;

        // Create default admin user if specified in environment
        if let (Ok(username), Ok(email), Ok(password)) = (
            env::var("ADMIN_USERNAME"),
            env::var("ADMIN_EMAIL"),
            env::var("ADMIN_PASSWORD"),
        ) {
            create_admin_user(&pool, &username, &email, &password).await?;
        }

        // Initialize services
        let auth_db = AuthDatabase::new(pool);
        let jwt_service = JwtService::new()?;
        let oauth_config = OAuthConfig::new()?;
        let oauth_service = OAuthService::new(oauth_config);

        let handlers = Arc::new(AuthHandlers::new(
            auth_db,
            jwt_service.clone(),
            oauth_service,
        ));

        // Create auth service for middleware
        let auth_service = AuthService::new(jwt_service);
        let rate_limiter = RateLimiter::new();

        // Build router with all authentication routes
        let router = Router::new()
            // Public routes (no authentication required)
            .route("/api/auth/register", post(crate::auth::handlers::register))
            .route("/api/auth/login", post(crate::auth::handlers::login))
            .route(
                "/api/auth/refresh",
                post(crate::auth::handlers::refresh_token),
            )
            .route(
                "/api/auth/reset-password",
                post(crate::auth::handlers::request_password_reset),
            )
            .route(
                "/api/auth/reset-password/confirm",
                post(crate::auth::handlers::confirm_password_reset),
            )
            .route(
                "/api/auth/verify-email/:token",
                get(crate::auth::handlers::verify_email),
            )
            // OAuth routes
            .route(
                "/api/auth/oauth/google",
                get(crate::auth::handlers::google_oauth_url),
            )
            .route(
                "/api/auth/oauth/google/callback",
                get(crate::auth::handlers::google_oauth_callback),
            )
            .route(
                "/api/auth/oauth/github",
                get(crate::auth::handlers::github_oauth_url),
            )
            .route(
                "/api/auth/oauth/github/callback",
                get(crate::auth::handlers::github_oauth_callback),
            )
            .route(
                "/api/auth/oauth/discord",
                get(crate::auth::handlers::discord_oauth_url),
            )
            .route(
                "/api/auth/oauth/discord/callback",
                get(crate::auth::handlers::discord_oauth_callback),
            )
            // Protected routes (authentication required)
            .route("/api/auth/logout", post(crate::auth::handlers::logout))
            .route("/api/auth/profile", get(crate::auth::handlers::get_profile))
            .route(
                "/api/auth/profile",
                put(crate::auth::handlers::update_profile),
            )
            .route(
                "/api/auth/change-password",
                post(crate::auth::handlers::change_password),
            )
            .layer(middleware::from_fn_with_state(
                auth_service.clone(),
                auth_middleware,
            ))
            // Admin only routes
            .route("/api/admin/users", get(get_all_users))
            .route("/api/admin/users/:id/deactivate", post(deactivate_user))
            .route("/api/admin/users/:id/activate", post(activate_user))
            .layer(middleware::from_fn(admin_only_middleware))
            .layer(middleware::from_fn_with_state(
                auth_service.clone(),
                auth_middleware,
            ))
            // Public info routes (optional authentication)
            .route("/api/info/health", get(health_check))
            .route("/api/info/stats", get(get_stats))
            .layer(middleware::from_fn_with_state(
                auth_service.clone(),
                optional_auth_middleware,
            ))
            // Add global middleware
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .allow_headers([AUTHORIZATION, CONTENT_TYPE]),
            )
            .with_state(handlers.clone());

        Ok(Self { router, handlers })
    }

    pub async fn start_server(self, port: u16) -> Result<(), AuthError> {
        let addr = format!("0.0.0.0:{}", port);
        info!("Authentication server starting on {}", addr);

        // Start background cleanup task
        let handlers_clone = self.handlers.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(3600)); // Run every hour
            loop {
                interval.tick().await;
                if let Err(e) = cleanup_expired_data(&handlers_clone.auth_db.pool).await {
                    error!("Failed to clean up expired data: {}", e);
                }
            }
        });

        // Start the server
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| AuthError::InternalServerError)?;

        axum::serve(listener, self.router)
            .await
            .map_err(|e| AuthError::InternalServerError)?;

        Ok(())
    }
}

// Admin endpoints
async fn get_all_users(
    State(handlers): State<Arc<AuthHandlers>>,
    _current_user: CurrentUser,
) -> Result<impl IntoResponse, AuthError> {
    // This would normally implement pagination
    // For now, just return a placeholder
    Ok(Json(json!({
        "users": [],
        "total": 0,
        "page": 1,
        "per_page": 20
    })))
}

async fn deactivate_user(
    State(handlers): State<Arc<AuthHandlers>>,
    _current_user: CurrentUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AuthError> {
    let user_id = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| AuthError::ValidationError("Invalid user ID".to_string()))?;

    handlers.auth_db.deactivate_user(user_id).await?;
    handlers.auth_db.delete_user_sessions(user_id).await?; // Force logout

    Ok(Json(json!({"message": "User deactivated successfully"})))
}

async fn activate_user(
    State(handlers): State<Arc<AuthHandlers>>,
    _current_user: CurrentUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AuthError> {
    let user_id = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| AuthError::ValidationError("Invalid user ID".to_string()))?;

    handlers.auth_db.activate_user(user_id).await?;

    Ok(Json(json!({"message": "User activated successfully"})))
}

// Public info endpoints
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "culiacan-rts-auth",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_stats(
    State(handlers): State<Arc<AuthHandlers>>,
    optional_user: OptionalCurrentUser,
) -> Result<impl IntoResponse, AuthError> {
    // Basic stats - could be expanded
    let stats = json!({
        "authenticated": optional_user.0.is_some(),
        "service": "culiacan-rts-auth",
        "features": {
            "registration": true,
            "oauth": {
                "google": true,
                "github": true,
                "discord": true
            },
            "email_verification": true,
            "password_reset": true,
            "role_based_access": true
        }
    });

    Ok(Json(stats))
}

// Helper function to start auth server in background
pub async fn start_auth_server_background(port: u16) -> Result<(), AuthError> {
    let server = AuthServer::new().await?;

    tokio::spawn(async move {
        if let Err(e) = server.start_server(port).await {
            error!("Auth server error: {}", e);
        }
    });

    info!(
        "Authentication server started in background on port {}",
        port
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let server = AuthServer::new().await.unwrap();

        let response = server
            .router
            .oneshot(
                Request::builder()
                    .uri("/api/info/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_protected_route_without_auth() {
        let server = AuthServer::new().await.unwrap();

        let response = server
            .router
            .oneshot(
                Request::builder()
                    .uri("/api/auth/profile")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

use crate::auth::{AuthError, JwtService, User, UserRole, extract_bearer_token, has_permission};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthService {
    pub jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(jwt_service: JwtService) -> Self {
        Self {
            jwt_service: Arc::new(jwt_service),
        }
    }
}

pub async fn auth_middleware(
    State(auth_service): State<AuthService>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(AuthError::UnauthorizedAccess)?;

    let token = extract_bearer_token(auth_header)?;
    let claims = auth_service.jwt_service.validate_token(token)?;

    // Add user information to request extensions for handlers to access
    request.extensions_mut().insert(CurrentUser {
        id: claims.sub.parse().map_err(|_| AuthError::InvalidToken)?,
        username: claims.username,
        email: claims.email,
        role: claims.role,
    });

    Ok(next.run(request).await)
}

pub async fn optional_auth_middleware(
    State(auth_service): State<AuthService>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(auth_header) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        if let Ok(token) = extract_bearer_token(auth_header) {
            if let Ok(claims) = auth_service.jwt_service.validate_token(token) {
                if let Ok(user_id) = claims.sub.parse() {
                    request.extensions_mut().insert(CurrentUser {
                        id: user_id,
                        username: claims.username,
                        email: claims.email,
                        role: claims.role,
                    });
                }
            }
        }
    }

    next.run(request).await
}

pub fn require_role(required_role: UserRole) -> impl Fn(Request, Next) -> Result<Response, AuthError> + Clone {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        async move {
            let current_user = request
                .extensions()
                .get::<CurrentUser>()
                .ok_or(AuthError::UnauthorizedAccess)?;

            if !has_permission(&current_user.role, &required_role) {
                return Err(AuthError::InsufficientPermissions);
            }

            Ok(next.run(request).await)
        }
    }
}

pub async fn admin_only_middleware(request: Request, next: Next) -> Result<Response, AuthError> {
    let current_user = request
        .extensions()
        .get::<CurrentUser>()
        .ok_or(AuthError::UnauthorizedAccess)?;

    if !matches!(current_user.role, UserRole::Admin) {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(request).await)
}

pub async fn moderator_or_admin_middleware(request: Request, next: Next) -> Result<Response, AuthError> {
    let current_user = request
        .extensions()
        .get::<CurrentUser>()
        .ok_or(AuthError::UnauthorizedAccess)?;

    if !matches!(current_user.role, UserRole::Admin | UserRole::Moderator) {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(request).await)
}

pub async fn verified_user_middleware(request: Request, next: Next) -> Result<Response, AuthError> {
    // This middleware would need access to the database to check if user is verified
    // For now, we'll just check if the user is authenticated
    let _current_user = request
        .extensions()
        .get::<CurrentUser>()
        .ok_or(AuthError::UnauthorizedAccess)?;

    // TODO: Add database check for user verification status
    // if !user_is_verified {
    //     return Err(AuthError::AccountNotVerified);
    // }

    Ok(next.run(request).await)
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
}

impl CurrentUser {
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }

    pub fn is_moderator_or_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Moderator)
    }

    pub fn has_permission_for(&self, required_role: &UserRole) -> bool {
        has_permission(&self.role, required_role)
    }
}

// Extractor for getting current user in handlers
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

#[axum::async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(AuthError::UnauthorizedAccess)
    }
}

// Optional user extractor that returns None if not authenticated
#[derive(Debug)]
pub struct OptionalCurrentUser(pub Option<CurrentUser>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalCurrentUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts.extensions.get::<CurrentUser>().cloned();
        Ok(OptionalCurrentUser(user))
    }
}

pub struct RateLimiter {
    // Simple in-memory rate limiter
    // In production, you'd want to use Redis or similar
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_rate_limit(&self, _key: &str) -> Result<(), AuthError> {
        // TODO: Implement actual rate limiting logic
        // For now, always allow
        Ok(())
    }
}

pub async fn rate_limit_middleware(
    State(_rate_limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract IP address or user ID for rate limiting
    let _key = "global"; // In reality, you'd use IP or user ID
    
    // Check rate limit
    // rate_limiter.check_rate_limit(key).await?;

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_user_permissions() {
        let admin_user = CurrentUser {
            id: uuid::Uuid::new_v4(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            role: UserRole::Admin,
        };

        let player_user = CurrentUser {
            id: uuid::Uuid::new_v4(),
            username: "player".to_string(),
            email: "player@example.com".to_string(),
            role: UserRole::Player,
        };

        assert!(admin_user.is_admin());
        assert!(admin_user.is_moderator_or_admin());
        assert!(admin_user.has_permission_for(&UserRole::Player));

        assert!(!player_user.is_admin());
        assert!(!player_user.is_moderator_or_admin());
        assert!(player_user.has_permission_for(&UserRole::Player));
        assert!(!player_user.has_permission_for(&UserRole::Admin));
    }
}
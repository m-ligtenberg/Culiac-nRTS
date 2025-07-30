use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    UserAlreadyExists,
    EmailAlreadyExists,
    UsernameAlreadyExists,
    InvalidToken,
    TokenExpired,
    InvalidRefreshToken,
    UnauthorizedAccess,
    InsufficientPermissions,
    AccountNotVerified,
    AccountDeactivated,
    ValidationError(String),
    DatabaseError(String),
    HashingError,
    TokenGenerationError,
    EmailSendError,
    OAuthError(String),
    InternalServerError,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid username/email or password"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::UserAlreadyExists => write!(f, "User already exists"),
            AuthError::EmailAlreadyExists => write!(f, "Email address is already registered"),
            AuthError::UsernameAlreadyExists => write!(f, "Username is already taken"),
            AuthError::InvalidToken => write!(f, "Invalid or malformed token"),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::InvalidRefreshToken => write!(f, "Invalid refresh token"),
            AuthError::UnauthorizedAccess => write!(f, "Unauthorized access"),
            AuthError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            AuthError::AccountNotVerified => write!(f, "Account email not verified"),
            AuthError::AccountDeactivated => write!(f, "Account has been deactivated"),
            AuthError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AuthError::HashingError => write!(f, "Password hashing error"),
            AuthError::TokenGenerationError => write!(f, "Token generation error"),
            AuthError::EmailSendError => write!(f, "Failed to send email"),
            AuthError::OAuthError(msg) => write!(f, "OAuth error: {}", msg),
            AuthError::InternalServerError => write!(f, "Internal server error"),
        }
    }
}

impl std::error::Error for AuthError {}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AuthError::EmailAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AuthError::UsernameAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::InvalidRefreshToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::UnauthorizedAccess => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, self.to_string()),
            AuthError::AccountNotVerified => (StatusCode::FORBIDDEN, self.to_string()),
            AuthError::AccountDeactivated => (StatusCode::FORBIDDEN, self.to_string()),
            AuthError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error occurred".to_string(),
            ),
            AuthError::HashingError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AuthError::TokenGenerationError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AuthError::EmailSendError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send email".to_string(),
            ),
            AuthError::OAuthError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AuthError::UserNotFound,
            _ => AuthError::DatabaseError(err.to_string()),
        }
    }
}

impl From<bcrypt::BcryptError> for AuthError {
    fn from(_: bcrypt::BcryptError) -> Self {
        AuthError::HashingError
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    }
}

impl From<validator::ValidationErrors> for AuthError {
    fn from(err: validator::ValidationErrors) -> Self {
        let error_messages: Vec<String> = err
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();

        AuthError::ValidationError(error_messages.join(", "))
    }
}

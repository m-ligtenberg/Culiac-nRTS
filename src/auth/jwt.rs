use crate::auth::{AuthError, Claims, User, UserRole};
use bevy::log::info;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
}

impl JwtService {
    pub fn new() -> Result<Self, AuthError> {
        let secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

        let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "culiacan-rts".to_string());

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            issuer,
            access_token_expiry: Duration::hours(1),
            refresh_token_expiry: Duration::days(30),
        })
    }

    pub fn generate_access_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiry = now + self.access_token_expiry;
        info!("Generating access token for user {}", user.id);

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: self.issuer.clone(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|_| AuthError::TokenGenerationError)
    }

    pub fn generate_refresh_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiry = now + self.refresh_token_expiry;

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: self.issuer.clone(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|_| AuthError::TokenGenerationError)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(AuthError::from)
    }

    pub fn extract_user_id_from_token(&self, token: &str) -> Result<Uuid, AuthError> {
        let claims = self.validate_token(token)?;
        Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)
    }

    pub fn generate_token_pair(&self, user: &User) -> Result<(String, String), AuthError> {
        let access_token = self.generate_access_token(user)?;
        let refresh_token = self.generate_refresh_token(user)?;
        Ok((access_token, refresh_token))
    }

    pub fn get_access_token_expiry_seconds(&self) -> i64 {
        self.access_token_expiry.num_seconds()
    }

    pub fn get_refresh_token_expiry_seconds(&self) -> i64 {
        self.refresh_token_expiry.num_seconds()
    }
}

impl Default for JwtService {
    fn default() -> Self {
        Self::new().expect("Failed to create JWT service")
    }
}

pub fn extract_bearer_token(auth_header: &str) -> Result<&str, AuthError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidToken);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix
    if token.is_empty() {
        return Err(AuthError::InvalidToken);
    }

    Ok(token)
}

pub fn has_permission(user_role: &UserRole, required_role: &UserRole) -> bool {
    match (user_role, required_role) {
        (UserRole::Admin, _) => true,
        (UserRole::Moderator, UserRole::Moderator | UserRole::Player | UserRole::Guest) => true,
        (UserRole::Player, UserRole::Player | UserRole::Guest) => true,
        (UserRole::Guest, UserRole::Guest) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            role: UserRole::Player,
            is_verified: true,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
        }
    }

    #[test]
    fn test_jwt_token_generation_and_validation() {
        let jwt_service = JwtService::new().unwrap();
        let user = create_test_user();

        let token = jwt_service.generate_access_token(&user).unwrap();
        let claims = jwt_service.validate_token(&token).unwrap();

        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.email, user.email);
    }

    #[test]
    fn test_bearer_token_extraction() {
        let auth_header = "Bearer abcdef123456";
        let token = extract_bearer_token(auth_header).unwrap();
        assert_eq!(token, "abcdef123456");

        let invalid_header = "Basic abcdef123456";
        assert!(extract_bearer_token(invalid_header).is_err());
    }

    #[test]
    fn test_permission_system() {
        assert!(has_permission(&UserRole::Admin, &UserRole::Guest));
        assert!(has_permission(&UserRole::Admin, &UserRole::Player));
        assert!(has_permission(&UserRole::Admin, &UserRole::Moderator));
        assert!(has_permission(&UserRole::Admin, &UserRole::Admin));

        assert!(has_permission(&UserRole::Moderator, &UserRole::Guest));
        assert!(has_permission(&UserRole::Moderator, &UserRole::Player));
        assert!(has_permission(&UserRole::Moderator, &UserRole::Moderator));
        assert!(!has_permission(&UserRole::Moderator, &UserRole::Admin));

        assert!(has_permission(&UserRole::Player, &UserRole::Guest));
        assert!(has_permission(&UserRole::Player, &UserRole::Player));
        assert!(!has_permission(&UserRole::Player, &UserRole::Moderator));
        assert!(!has_permission(&UserRole::Player, &UserRole::Admin));

        assert!(has_permission(&UserRole::Guest, &UserRole::Guest));
        assert!(!has_permission(&UserRole::Guest, &UserRole::Player));
        assert!(!has_permission(&UserRole::Guest, &UserRole::Moderator));
        assert!(!has_permission(&UserRole::Guest, &UserRole::Admin));
    }
}

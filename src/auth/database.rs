use crate::auth::{
    AuthError, CreateUserRequest, EmailVerification, OAuthProvider, PasswordReset, Session, User,
    UserRole,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use bevy::log::info;
use chrono::{DateTime, Duration, Utc};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct AuthDatabase {
    pub pool: Pool<Sqlite>,
}

impl AuthDatabase {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, AuthError> {
        let user_id = Uuid::new_v4();
        let password_hash = hash(&request.password, DEFAULT_COST)?;
        let now = Utc::now();

        // Check if username or email already exists
        if self.username_exists(&request.username).await? {
            return Err(AuthError::UsernameAlreadyExists);
        }

        if self.email_exists(&request.email).await? {
            return Err(AuthError::EmailAlreadyExists);
        }

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, is_verified, is_active, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(UserRole::Player)
        .bind(false) // not verified by default
        .bind(true)  // active by default
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        info!("User '{}' created successfully", user.username);

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AuthError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User, AuthError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?1")
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, AuthError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?1")
            .bind(email)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_identifier(&self, identifier: &str) -> Result<User, AuthError> {
        // Try to find user by username or email
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?1 OR email = ?1")
                .bind(identifier)
                .fetch_one(&self.pool)
                .await?;

        Ok(user)
    }

    pub async fn verify_password(&self, user: &User, password: &str) -> Result<bool, AuthError> {
        verify(password, &user.password_hash).map_err(AuthError::from)
    }

    pub async fn update_last_login(&self, user_id: Uuid) -> Result<(), AuthError> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_login = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(now)
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn username_exists(&self, username: &str) -> Result<bool, AuthError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = ?1")
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 > 0)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool, AuthError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = ?1")
            .bind(email)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 > 0)
    }

    pub async fn update_user_email(&self, user_id: Uuid, email: &str) -> Result<(), AuthError> {
        if self.email_exists(email).await? {
            return Err(AuthError::EmailAlreadyExists);
        }

        let now = Utc::now();
        sqlx::query(
            "UPDATE users SET email = ?1, updated_at = ?2, is_verified = false WHERE id = ?3",
        )
        .bind(email)
        .bind(now)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_username(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<(), AuthError> {
        if self.username_exists(username).await? {
            return Err(AuthError::UsernameAlreadyExists);
        }

        let now = Utc::now();
        sqlx::query("UPDATE users SET username = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(username)
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_user_password(
        &self,
        user_id: Uuid,
        new_password: &str,
    ) -> Result<(), AuthError> {
        let password_hash = hash(new_password, DEFAULT_COST)?;
        let now = Utc::now();

        sqlx::query("UPDATE users SET password_hash = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(password_hash)
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn verify_user_email(&self, user_id: Uuid) -> Result<(), AuthError> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET is_verified = true, updated_at = ?1 WHERE id = ?2")
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn deactivate_user(&self, user_id: Uuid) -> Result<(), AuthError> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET is_active = false, updated_at = ?1 WHERE id = ?2")
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn activate_user(&self, user_id: Uuid) -> Result<(), AuthError> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET is_active = true, updated_at = ?1 WHERE id = ?2")
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Session Management
    pub async fn create_session(
        &self,
        user_id: Uuid,
        refresh_token: &str,
        expires_at: DateTime<Utc>,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<Session, AuthError> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (id, user_id, refresh_token, expires_at, created_at, last_used, user_agent, ip_address)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            RETURNING *
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(refresh_token)
        .bind(expires_at)
        .bind(now)
        .bind(now)
        .bind(user_agent)
        .bind(ip_address)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_session_by_token(&self, refresh_token: &str) -> Result<Session, AuthError> {
        let session =
            sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE refresh_token = ?1")
                .bind(refresh_token)
                .fetch_one(&self.pool)
                .await?;

        Ok(session)
    }

    pub async fn update_session_last_used(&self, session_id: Uuid) -> Result<(), AuthError> {
        let now = Utc::now();
        sqlx::query("UPDATE sessions SET last_used = ?1 WHERE id = ?2")
            .bind(now)
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_session(&self, session_id: Uuid) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM sessions WHERE id = ?1")
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_expired_sessions(&self) -> Result<(), AuthError> {
        let now = Utc::now();
        sqlx::query("DELETE FROM sessions WHERE expires_at < ?1")
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_user_sessions(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Password Reset
    pub async fn create_password_reset(
        &self,
        user_id: Uuid,
        token: &str,
    ) -> Result<PasswordReset, AuthError> {
        let reset_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::hours(24); // Reset token expires in 24 hours

        let reset = sqlx::query_as::<_, PasswordReset>(
            r#"
            INSERT INTO password_resets (id, user_id, token, expires_at, used, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            RETURNING *
            "#,
        )
        .bind(reset_id)
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .bind(false)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(reset)
    }

    pub async fn get_password_reset_by_token(
        &self,
        token: &str,
    ) -> Result<PasswordReset, AuthError> {
        let reset = sqlx::query_as::<_, PasswordReset>(
            "SELECT * FROM password_resets WHERE token = ?1 AND used = false AND expires_at > ?2",
        )
        .bind(token)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(reset)
    }

    pub async fn mark_password_reset_used(&self, reset_id: Uuid) -> Result<(), AuthError> {
        sqlx::query("UPDATE password_resets SET used = true WHERE id = ?1")
            .bind(reset_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Email Verification
    pub async fn create_email_verification(
        &self,
        user_id: Uuid,
        token: &str,
    ) -> Result<EmailVerification, AuthError> {
        let verification_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::hours(48); // Verification token expires in 48 hours

        let verification = sqlx::query_as::<_, EmailVerification>(
            r#"
            INSERT INTO email_verifications (id, user_id, token, expires_at, verified, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            RETURNING *
            "#,
        )
        .bind(verification_id)
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .bind(false)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(verification)
    }

    pub async fn get_email_verification_by_token(
        &self,
        token: &str,
    ) -> Result<EmailVerification, AuthError> {
        let verification = sqlx::query_as::<_, EmailVerification>(
            "SELECT * FROM email_verifications WHERE token = ?1 AND verified = false AND expires_at > ?2"
        )
        .bind(token)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(verification)
    }

    pub async fn mark_email_verification_used(
        &self,
        verification_id: Uuid,
    ) -> Result<(), AuthError> {
        sqlx::query("UPDATE email_verifications SET verified = true WHERE id = ?1")
            .bind(verification_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // OAuth Provider Management
    pub async fn create_oauth_provider(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_user_id: &str,
        access_token: Option<String>,
        refresh_token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<OAuthProvider, AuthError> {
        let provider_id = Uuid::new_v4();
        let now = Utc::now();

        let oauth_provider = sqlx::query_as::<_, OAuthProvider>(
            r#"
            INSERT INTO oauth_providers (id, user_id, provider, provider_user_id, access_token, refresh_token, expires_at, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            RETURNING *
            "#,
        )
        .bind(provider_id)
        .bind(user_id)
        .bind(provider)
        .bind(provider_user_id)
        .bind(access_token)
        .bind(refresh_token)
        .bind(expires_at)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(oauth_provider)
    }

    pub async fn get_oauth_provider(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<OAuthProvider, AuthError> {
        let oauth_provider = sqlx::query_as::<_, OAuthProvider>(
            "SELECT * FROM oauth_providers WHERE provider = ?1 AND provider_user_id = ?2",
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(oauth_provider)
    }

    pub async fn get_user_oauth_providers(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<OAuthProvider>, AuthError> {
        let providers =
            sqlx::query_as::<_, OAuthProvider>("SELECT * FROM oauth_providers WHERE user_id = ?1")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?;

        Ok(providers)
    }
}

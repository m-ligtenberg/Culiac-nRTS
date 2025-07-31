use crate::auth::AuthError;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
use std::env;

pub async fn initialize_database() -> Result<Pool<Sqlite>, AuthError> {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:culiacan_rts.db".to_string());

    // Create database if it doesn't exist
    if !Sqlite::database_exists(&database_url)
        .await
        .unwrap_or(false)
    {
        info!("Creating database {}", database_url);
        match Sqlite::create_database(&database_url).await {
            Ok(_) => info!("Database created successfully"),
            Err(error) => {
                return Err(AuthError::DatabaseError(format!(
                    "Failed to create database: {}",
                    error
                )))
            }
        }
    }

    // Connect to the database
    let pool = SqlitePool::connect(&database_url)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to connect to database: {}", e)))?;

    // Run migrations
    run_migrations(&pool).await?;

    println!("Database initialized successfully");
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), AuthError> {
    info!("Running database migrations...");

    // Read and execute migration files
    let migrations = [
        include_str!("../../migrations/001_create_users_table.sql"),
        include_str!("../../migrations/002_create_sessions_table.sql"),
        include_str!("../../migrations/003_create_password_resets_table.sql"),
        include_str!("../../migrations/004_create_email_verifications_table.sql"),
        include_str!("../../migrations/005_create_oauth_providers_table.sql"),
    ];

    for (i, migration) in migrations.iter().enumerate() {
        info!("Running migration {}...", i + 1);
        sqlx::query(migration)
            .execute(pool)
            .await
            .map_err(|e| AuthError::DatabaseError(format!("Migration {} failed: {}", i + 1, e)))?;
    }

    info!("All migrations completed successfully");
    Ok(())
}

pub async fn cleanup_expired_data(pool: &SqlitePool) -> Result<(), AuthError> {
    info!("Cleaning up expired data...");

    // Clean up expired sessions
    let expired_sessions = sqlx::query("DELETE FROM sessions WHERE expires_at < datetime('now')")
        .execute(pool)
        .await
        .map_err(|e| {
            AuthError::DatabaseError(format!("Failed to clean expired sessions: {}", e))
        })?;

    info!("Cleaned up {} expired sessions", expired_sessions.rows_affected());

    // Clean up expired password resets
    let expired_resets =
        sqlx::query("DELETE FROM password_resets WHERE expires_at < datetime('now')")
            .execute(pool)
            .await
            .map_err(|e| {
                AuthError::DatabaseError(format!("Failed to clean expired password resets: {}", e))
            })?;

    println!(
        "Cleaned up {} expired password resets",
        expired_resets.rows_affected()
    );

    // Clean up expired email verifications
    let expired_verifications =
        sqlx::query("DELETE FROM email_verifications WHERE expires_at < datetime('now')")
            .execute(pool)
            .await
            .map_err(|e| {
                AuthError::DatabaseError(format!(
                    "Failed to clean expired email verifications: {}",
                    e
                ))
            })?;

    println!(
        "Cleaned up {} expired email verifications",
        expired_verifications.rows_affected()
    );

    Ok(())
}

pub async fn create_admin_user(
    pool: &SqlitePool,
    username: &str,
    email: &str,
    password: &str,
) -> Result<(), AuthError> {
    use crate::auth::{AuthDatabase, CreateUserRequest, UserRole};
    use bcrypt::{hash, DEFAULT_COST};
    use chrono::Utc;
    use uuid::Uuid;

    let auth_db = AuthDatabase::new(pool.clone());

    // Check if admin user already exists
    if auth_db.get_user_by_username(username).await.is_ok() {
        info!("Admin user '{}' already exists", username);
        return Ok(());
    }

    // Create admin user directly in database (bypass normal registration)
    let user_id = Uuid::new_v4();
    let password_hash = hash(password, DEFAULT_COST).map_err(|_| AuthError::HashingError)?;
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, role, is_verified, is_active, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
    )
    .bind(user_id.to_string())
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind("admin")
    .bind(true)  // Admin is pre-verified
    .bind(true)  // Admin is active
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to create admin user: {}", e)))?;

    info!("Admin user '{}' created successfully", username);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_database() {
        // Use in-memory database for testing
        std::env::set_var("DATABASE_URL", "sqlite::memory:");

        let pool = initialize_database().await.unwrap();

        // Verify tables were created
        let tables: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .fetch_all(&pool)
                .await
                .unwrap();

        let table_names: Vec<String> = tables.into_iter().map(|(name,)| name).collect();

        assert!(table_names.contains(&"users".to_string()));
        assert!(table_names.contains(&"sessions".to_string()));
        assert!(table_names.contains(&"password_resets".to_string()));
        assert!(table_names.contains(&"email_verifications".to_string()));
        assert!(table_names.contains(&"oauth_providers".to_string()));
    }

    #[tokio::test]
    async fn test_create_admin_user() {
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        let pool = initialize_database().await.unwrap();

        create_admin_user(&pool, "admin", "admin@example.com", "admin123")
            .await
            .unwrap();

        // Verify admin user was created
        let user: (String, String, String) =
            sqlx::query_as("SELECT username, email, role FROM users WHERE username = 'admin'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(user.0, "admin");
        assert_eq!(user.1, "admin@example.com");
        assert_eq!(user.2, "admin");
    }
}

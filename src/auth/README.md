# CuliacanRTS Authentication System

A comprehensive authentication and authorization system built for the CuliacanRTS game using Rust, Axum, SQLx, and JWT tokens.

## Features

### ✅ Core Features Implemented

- **User Registration & Login** - Secure user accounts with bcrypt password hashing
- **JWT Authentication** - Stateless authentication with access and refresh tokens
- **Session Management** - Track user sessions with automatic cleanup
- **Role-Based Access Control** - Admin, Moderator, Player, and Guest roles
- **OAuth 2.0 Integration** - Google, GitHub, and Discord OAuth providers
- **Password Reset** - Secure password reset via email tokens
- **Email Verification** - Email verification system for new accounts
- **Security Features** - Rate limiting, CORS, input validation
- **Database Management** - SQLite with automatic migrations
- **API Documentation** - Comprehensive REST API

## Architecture

### Modules

- `models.rs` - User models, requests, and database schemas
- `handlers.rs` - HTTP request handlers for all endpoints
- `middleware.rs` - Authentication middleware and extractors
- `jwt.rs` - JWT token generation and validation
- `database.rs` - Database operations and queries
- `oauth.rs` - OAuth 2.0 provider integrations
- `errors.rs` - Comprehensive error handling
- `db_init.rs` - Database initialization and migrations
- `server.rs` - Web server setup and routing
- `integration.rs` - Game integration helpers

### Database Schema

- `users` - User accounts and profiles
- `sessions` - Active user sessions with refresh tokens
- `password_resets` - Password reset tokens
- `email_verifications` - Email verification tokens
- `oauth_providers` - OAuth provider linkages

## Quick Start

### 1. Environment Setup

```bash
# Required
export JWT_SECRET="your-super-secret-jwt-key"
export DATABASE_URL="sqlite:culiacan_rts_auth.db"

# Optional - Admin user
export ADMIN_USERNAME="admin"
export ADMIN_EMAIL="admin@example.com"  
export ADMIN_PASSWORD="secure_password"

# Optional - OAuth providers
export GOOGLE_CLIENT_ID="your-google-client-id"
export GOOGLE_CLIENT_SECRET="your-google-client-secret"
export GOOGLE_REDIRECT_URL="http://localhost:8080/api/auth/oauth/google/callback"

export GITHUB_CLIENT_ID="your-github-client-id"
export GITHUB_CLIENT_SECRET="your-github-client-secret"
export GITHUB_REDIRECT_URL="http://localhost:8080/api/auth/oauth/github/callback"

export DISCORD_CLIENT_ID="your-discord-client-id" 
export DISCORD_CLIENT_SECRET="your-discord-client-secret"
export DISCORD_REDIRECT_URL="http://localhost:8080/api/auth/oauth/discord/callback"
```

### 2. Integration with Game

#### Option A: Automatic Integration

```rust
// In your main.rs
use crate::auth::setup_game_authentication;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize authentication system
    let _auth_integration = setup_game_authentication().await?;
    
    // Start your Bevy game
    App::new()
        // ... your game setup
        .run();
    
    Ok(())
}
```

#### Option B: Manual Integration

```rust
use crate::auth::{GameAuthIntegration, AuthServer};

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Custom auth setup
    let integration = GameAuthIntegration::new();
    integration.initialize().await?;
    
    // Or start server directly
    let server = AuthServer::new().await?;
    tokio::spawn(async move {
        server.start_server(8080).await.unwrap();
    });
    
    // Start game...
    Ok(())
}
```

### 3. API Usage Examples

#### User Registration
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "player1",
    "email": "player1@example.com",
    "password": "secure_password"
  }'
```

#### User Login
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "player1",
    "password": "secure_password"
  }'
```

#### Protected Request
```bash
curl -X GET http://localhost:8080/api/auth/profile \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### OAuth Login
```bash
# Get OAuth URL
curl http://localhost:8080/api/auth/oauth/google

# Redirect user to returned URL, then handle callback
# GET /api/auth/oauth/google/callback?code=AUTH_CODE&state=STATE
```

## API Endpoints

### Public Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | User registration |
| POST | `/api/auth/login` | User login |
| POST | `/api/auth/refresh` | Refresh access token |
| POST | `/api/auth/reset-password` | Request password reset |
| POST | `/api/auth/reset-password/confirm` | Confirm password reset |
| GET | `/api/auth/verify-email/:token` | Verify email address |
| GET | `/api/auth/oauth/google` | Google OAuth URL |
| GET | `/api/auth/oauth/google/callback` | Google OAuth callback |
| GET | `/api/auth/oauth/github` | GitHub OAuth URL |
| GET | `/api/auth/oauth/github/callback` | GitHub OAuth callback |
| GET | `/api/auth/oauth/discord` | Discord OAuth URL |
| GET | `/api/auth/oauth/discord/callback` | Discord OAuth callback |
| GET | `/api/info/health` | Health check |
| GET | `/api/info/stats` | Service statistics |

### Protected Endpoints (Requires Authentication)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/logout` | User logout |
| GET | `/api/auth/profile` | Get user profile |
| PUT | `/api/auth/profile` | Update user profile |
| POST | `/api/auth/change-password` | Change password |

### Admin Endpoints (Requires Admin Role)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/admin/users` | List all users |
| POST | `/api/admin/users/:id/deactivate` | Deactivate user |
| POST | `/api/admin/users/:id/activate` | Activate user |

## Request/Response Examples

### Registration Response
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "player1",
    "email": "player1@example.com",
    "role": "player",
    "is_verified": false,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login": null
  }
}
```

### Error Response
```json
{
  "error": "Invalid username/email or password",
  "status": 401
}
```

## Security Features

- **Password Hashing** - bcrypt with configurable cost
- **JWT Tokens** - Secure token-based authentication
- **Rate Limiting** - Prevent brute force attacks
- **CORS Protection** - Cross-origin request security
- **Input Validation** - Comprehensive request validation
- **Session Management** - Automatic token cleanup
- **Role-Based Access** - Granular permission system

## Role Hierarchy

1. **Admin** - Full system access
2. **Moderator** - User management, content moderation
3. **Player** - Standard game access
4. **Guest** - Limited/read-only access

## Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test auth::tests

# Run integration tests
cargo test --test integration

# Run with output
cargo test -- --nocapture
```

## Development

### Adding New Endpoints

1. Define request/response models in `models.rs`
2. Implement handler function in `handlers.rs`
3. Add route to `server.rs`
4. Add appropriate middleware if needed
5. Write tests

### Adding New OAuth Providers

1. Add provider config to `oauth.rs`
2. Implement provider-specific user info struct
3. Add authorization URL and callback handlers
4. Update `handlers.rs` with new endpoints
5. Add routes to `server.rs`

### Database Migrations

1. Create new `.sql` file in `migrations/`
2. Update `db_init.rs` to include new migration
3. Test migration with fresh database

## Production Deployment

### Environment Variables (Required)
```bash
export JWT_SECRET="your-production-secret"
export DATABASE_URL="sqlite:production.db"
export AUTH_SERVER_PORT="8080"
```

### Environment Variables (Optional)
```bash
export ADMIN_USERNAME="admin"
export ADMIN_EMAIL="admin@yourdomain.com"
export ADMIN_PASSWORD="secure_admin_password"

# OAuth (as needed)
export GOOGLE_CLIENT_ID="..."
export GOOGLE_CLIENT_SECRET="..."
# ... etc
```

### Security Checklist

- [ ] Use strong JWT secret (32+ characters)
- [ ] Enable HTTPS in production
- [ ] Configure proper CORS origins
- [ ] Set up rate limiting
- [ ] Regular database backups
- [ ] Monitor for suspicious activity
- [ ] Update dependencies regularly

## Troubleshooting

### Common Issues

**Q: "JWT_SECRET not found" error**
A: Set the JWT_SECRET environment variable

**Q: Database connection failed** 
A: Check DATABASE_URL and file permissions

**Q: OAuth callback errors**
A: Verify redirect URLs match exactly

**Q: Token expired errors**
A: Use refresh token to get new access token

### Logs and Debugging

The system provides detailed logging for:
- Authentication attempts
- Token generation/validation
- Database operations
- OAuth flows
- Error conditions

## Contributing

1. Follow Rust coding standards
2. Add tests for new functionality
3. Update documentation
4. Use meaningful commit messages
5. Follow security best practices

## License

This authentication system is part of the CuliacanRTS project and follows the same licensing terms.
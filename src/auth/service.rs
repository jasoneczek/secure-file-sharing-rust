use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::passwords::{hash_password, verify_password};
use crate::auth::repository::AuthUserRepository;
use crate::auth::token::create_token;
use crate::auth::types::{AuthTokenResponse, LoginRequest, RegisterRequest};

#[async_trait]
pub trait AuthService {
    async fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, String>;
    async fn login(&self, req: LoginRequest) -> Result<AuthTokenResponse, String>;
    async fn refresh(&self, refresh_token: String) -> Result<AuthTokenResponse, String>;
}

const EXPIRES_IN: u64 = 3600;

#[derive(Clone)]
pub struct SimpleAuthService {
    pub repo: AuthUserRepository,
}

impl SimpleAuthService {
    pub fn new(repo: AuthUserRepository) -> Self {
        Self { repo }
    }

    /// Issue access + refresh tokens and store refresh token in DB
    async fn issue_tokens(&self, user_id: u32) -> Result<AuthTokenResponse, String> {
        let access_token = create_token(user_id).map_err(|_| "Token creation failed")?;
        let refresh_token = Uuid::new_v4().to_string();

        self.repo
            .revoke_all_refresh_tokens_for_user(user_id)
            .await
            .map_err(|_| "Database error")?;

        self.repo
            .insert_refresh_token(user_id, &refresh_token)
            .await
            .map_err(|_| "Database error")?;

        Ok(AuthTokenResponse {
            access_token,
            refresh_token,
            expires_in: EXPIRES_IN,
        })
    }
}

#[async_trait]
impl AuthService for SimpleAuthService {
    async fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, String> {
        // Basic input validation
        if req.username.trim().is_empty() {
            return Err("Username cannot be empty".into());
        }

        if req.password.len() < 8 {
            return Err("Password must be at least 8 characters".into());
        }

        // Check if unique
        let existing = self
            .repo
            .find_by_username(&req.username)
            .await
            .map_err(|_| "Database error")?;

        if existing.is_some() {
            return Err("Username already exists".into());
        }

        // Hash password
        let password_hash = hash_password(&req.password).map_err(|_| "Password hashing failed")?;

        // Create user
        let user = self
            .repo
            .create(req.username, password_hash)
            .await
            .map_err(|_| "Database error")?;

        // Issue access + refresh tokens
        self.issue_tokens(user.id).await
    }

    async fn login(&self, req: LoginRequest) -> Result<AuthTokenResponse, String> {
        // Find user
        let user = self
            .repo
            .find_by_username(&req.username)
            .await
            .map_err(|_| "Database error")?
            .ok_or("Invalid credentials")?;

        // Verify password
        let valid = verify_password(&req.password, &user.password_hash)
            .map_err(|_| "Password verification failed")?;

        if !valid {
            return Err("Invalid credentials".into());
        }

        // Issue access + refresh tokens
        self.issue_tokens(user.id).await
    }

    async fn refresh(&self, refresh_token: String) -> Result<AuthTokenResponse, String> {
        let new_refresh = Uuid::new_v4().to_string();

        let user_id = self
            .repo
            .rotate_refresh_token(&refresh_token, &new_refresh)
            .await
            .map_err(|_| "Database error")?
            .ok_or("Invalid refresh token")?;

        let access_token = create_token(user_id).map_err(|_| "Token creation failed")?;

        Ok(AuthTokenResponse {
            access_token,
            refresh_token: new_refresh,
            expires_in: EXPIRES_IN,
        })
    }
}

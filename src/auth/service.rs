use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::passwords::{hash_password, verify_password};
use crate::auth::repository::AuthUserRepository;
use crate::auth::token::create_token;
use crate::auth::types::{AuthTokenResponse, LoginRequest, RegisterRequest};

/// Auth errors split into:
/// - user mistakes (400/401)
/// - unexpected internal failures (500)
#[derive(Debug)]
pub enum AuthError {
    BadRequest(&'static str),
    Unauthorized(&'static str),
    Internal,
}

impl AuthError {
    pub fn bad_request(msg: &'static str) -> Self {
        AuthError::BadRequest(msg)
    }

    pub fn unauthorized(msg: &'static str) -> Self {
        AuthError::Unauthorized(msg)
    }
}

#[async_trait]
pub trait AuthService {
    async fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, AuthError>;
    async fn login(&self, req: LoginRequest) -> Result<AuthTokenResponse, AuthError>;
    async fn refresh(&self, refresh_token: String) -> Result<AuthTokenResponse, AuthError>;
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

    /// Log internal debug info
    fn internal<E: std::fmt::Debug>(e: E) -> AuthError {
        eprintln!("[internal] {e:?}");
        AuthError::Internal
    }

    /// Issue access + refresh tokens and store refresh token in DB
    async fn issue_tokens(
        &self,
        user_id: u32,
        username: &str,
    ) -> Result<AuthTokenResponse, AuthError> {
        let access_token = create_token(user_id, username)
            .map_err(Self::internal)?;

        let refresh_token = Uuid::new_v4().to_string();

        self.repo
            .revoke_all_refresh_tokens_for_user(user_id)
            .await
            .map_err(Self::internal)?;

        self.repo
            .insert_refresh_token(user_id, &refresh_token)
            .await
            .map_err(Self::internal)?;

        Ok(AuthTokenResponse {
            access_token,
            refresh_token,
            expires_in: EXPIRES_IN,
        })
    }
}

#[async_trait]
impl AuthService for SimpleAuthService {
    async fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, AuthError> {
        if req.username.trim().is_empty() {
            return Err(AuthError::bad_request("Username cannot be empty"));
        }

        if req.password.len() < 8 {
            return Err(AuthError::bad_request(
                "Password must be at least 8 characters",
            ));
        }

        let existing = self
            .repo
            .find_by_username(&req.username)
            .await
            .map_err(Self::internal)?;

        if existing.is_some() {
            return Err(AuthError::bad_request("Username already exists"));
        }

        let password_hash = hash_password(&req.password).map_err(Self::internal)?;

        let user = self
            .repo
            .create(req.username, password_hash)
            .await
            .map_err(Self::internal)?;

        self.issue_tokens(user.id, &user.username).await
    }

    async fn login(&self, req: LoginRequest) -> Result<AuthTokenResponse, AuthError> {
        let user = self
            .repo
            .find_by_username(&req.username)
            .await
            .map_err(Self::internal)?
            .ok_or(AuthError::unauthorized("Invalid credentials"))?;

        let valid = verify_password(&req.password, &user.password_hash).map_err(Self::internal)?;

        if !valid {
            return Err(AuthError::unauthorized("Invalid credentials"));
        }

        self.issue_tokens(user.id, &user.username).await
    }

    async fn refresh(&self, refresh_token: String) -> Result<AuthTokenResponse, AuthError> {
        let new_refresh = Uuid::new_v4().to_string();

        let user_id = self
            .repo
            .rotate_refresh_token(&refresh_token, &new_refresh)
            .await
            .map_err(Self::internal)?
            .ok_or(AuthError::unauthorized("Invalid refresh token"))?;

        let username = self
            .repo
            .get_username_by_id(user_id)
            .await
            .map_err(Self::internal)?
            .ok_or(AuthError::unauthorized("Invalid refresh token"))?;

        let access_token = create_token(user_id, &username).map_err(Self::internal)?;

        Ok(AuthTokenResponse {
            access_token,
            refresh_token: new_refresh,
            expires_in: EXPIRES_IN,
        })
    }
}

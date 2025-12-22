use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use uuid::Uuid;

use crate::auth::passwords::{hash_password, verify_password};
use crate::auth::repository::AuthUserRepository;
use crate::auth::token::create_token;
use crate::auth::types::{AuthTokenResponse, LoginRequest, RegisterRequest};

pub trait AuthService {
    fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, String>;
    fn login(&self, req: LoginRequest) -> Result<AuthTokenResponse, String>;
    fn refresh(&self, refresh_token: String) -> Result<AuthTokenResponse, String>;
}

const EXPIRES_IN: u64 = 3600;

#[derive(Clone)]
pub struct SimpleAuthService {
    pub repo: AuthUserRepository,

    /// refresh_token -> user_id (in memory for now, move to DB)
    refresh_tokens: Arc<Mutex<HashMap<String, u32>>>,
}

impl SimpleAuthService {
    pub fn new(repo: AuthUserRepository) -> Self {
        Self {
            repo,
            refresh_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Issue access + refresh tokens and store refresh token mapping
    fn issue_tokens(&self, user_id: u32) -> Result<AuthTokenResponse, String> {
        let access_token = create_token(user_id).map_err(|_| "Token creation failed")?;
        let refresh_token = Uuid::new_v4().to_string();

        {
            let mut map = self.refresh_tokens.lock();
            map.insert(refresh_token.clone(), user_id);
        }

        Ok(AuthTokenResponse {
            access_token,
            refresh_token,
            expires_in: EXPIRES_IN,
        })
    }
}

impl AuthService for SimpleAuthService {
    fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, String> {
        // Basic input validation
        if req.username.trim().is_empty() {
            return Err("Username cannot be empty".into());
        }

        if req.password.len() < 8 {
            return Err("Password must be at least 8 characters".into());
        }

        // Check if unique
        if self.repo.find_by_username(&req.username).is_some() {
            return Err("Username already exists".into());
        }

        // Hash password
        let password_hash = hash_password(&req.password).map_err(|_| "Password hashing failed")?;

        // Create user
        let user = self.repo.create(req.username, password_hash);

        // Issue access + refresh tokens
        self.issue_tokens(user.id)
    }

    fn login(&self, req: LoginRequest) -> Result<AuthTokenResponse, String> {
        // Find user
        let user = self
            .repo
            .find_by_username(&req.username)
            .ok_or("Invalid credentials")?;

        // Verify password
        let valid = verify_password(&req.password, &user.password_hash)
            .map_err(|_| "Password verification failed")?;

        if !valid {
            return Err("Invalid credentials".into());
        }

        // Issue access + refresh tokens
        self.issue_tokens(user.id)
    }

    fn refresh(&self, refresh_token: String) -> Result<AuthTokenResponse, String> {
        // Look up refresh token
        let user_id = {
            let map = self.refresh_tokens.lock();
            *map.get(&refresh_token).ok_or("Invalid refresh token")?
        };

        // Rotate: remove the old refresh token so it can't be reused
        {
            let mut map = self.refresh_tokens.lock();
            map.remove(&refresh_token);
        }

        // Issue new access + refresh tokens
        self.issue_tokens(user_id)
    }
}

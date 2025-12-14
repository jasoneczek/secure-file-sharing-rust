use crate::auth::types::{RegisterRequest, LoginRequest, AuthTokenResponse};
use crate::auth::passwords::{hash_password, verify_password};
use crate::auth::repository::AuthUserRepository;

pub trait AuthService {
    fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, String>;
    fn login(&self, req: LoginRequest) -> Result<AuthTokenResponse, String>;
}

#[derive(Clone)]
pub struct SimpleAuthService {
    pub repo: AuthUserRepository,
}

impl SimpleAuthService {
    pub fn new(repo: AuthUserRepository) -> Self {
        Self { repo }
    }
}

impl AuthService for SimpleAuthService {
    fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, String> {
        // Basic input validation
        if req.username.trim().is_empty() {
            return Err("Username cannot be empty".into())
        }

        if req.password.len() < 8 {
            return Err("Password must be at least 8 characters".into());
        }

        // Check if unique
        if self.repo.find_by_username(&req.username).is_some() {
            return Err("Username already exists".into());
        }

        // Hash password
         let password_hash =
            hash_password(&req.password).map_err(|_| "Password hashing failed")?;

         // Create user
         let user = self.repo.create(req.username, password_hash);

         // Temporary token
         Ok(AuthTokenResponse {
             access_token: format!("fake-token-user-{}", user.id),
             expires_in: 3600,
         })
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

        // Issue token
        Ok(AuthTokenResponse {
            access_token: format!("fake-token-user-{}", user.id),
            expires_in: 3600,
        })
    }
}
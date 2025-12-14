use crate::auth::types::{RegisterRequest, AuthTokenResponse};
use crate::auth::passwords::hash_password;
use crate::auth::repository::AuthUserRepository;

pub trait AuthService {
    fn register(&self, req: RegisterRequest) -> Result<AuthTokenResponse, String>;
    fn login(&self, req: crate::auth::types::LoginRequest) -> Result<AuthTokenResponse, String>;
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

    fn login(&self, _req: crate::auth::types::LoginRequest) -> Result<AuthTokenResponse, String> {
        Err("not implemented".into())
    }
}
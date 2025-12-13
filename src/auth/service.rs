/// Authentication service interface.
///
/// Defines the core authentication operations used by HTTP handlers.
/// Implementations handle user registration and login logic.
pub trait AuthService {
    /// Register a new user and return authentication token data.
    fn register(&self, _req: RegisterRequest) -> Result<AuthTokenResponse, String>;
    /// Authenticate an existing user and return authentication token data.
    fn login(&self, _req: LoginRequest) -> Result<AuthTokenResponse, String>;
}
/// Simple in-memory implementation of `AuthService`.
///
/// This implementation is used for development and testing and stores
/// users in an in-memory repository.
#[derive(Clone)]
pub struct SimpleAuthService {
    pub repo: AuthUserRepository,
}

impl SimpleAuthService {
    /// Create a new `SimpleAuthService` backed by the given repository.
    pub fn new(repo: AuthUserRepository) -> Self {
        Self { repo }
    }
}

impl AuthService for SimpleAuthService {
    fn register(&self, _req: RegisterRequest) -> Result<AuthTokenResponse, String> {
        Err("not implemented".into())
    }

    fn login(&self, _req: LoginRequest) -> Result<AuthTokenResponse, String> {
        Err("not implemented".into())
    }
}
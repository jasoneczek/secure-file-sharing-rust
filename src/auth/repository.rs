/// In-memory repository for authentication users.
///
/// This repository will store users and their password hashes
/// for registration and login. It is intended for development
/// and will be backed by a real database in a later stage.
#[derive(Clone)]
pub struct AuthUserRepository;

impl AuthUserRepository {
    /// Create a new authentication user repository.
    pub fn new() -> Self {
        Self
    }
}
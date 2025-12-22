use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::auth::types::AuthUser;

/// In-memory repository for authentication users
#[derive(Clone)]
pub struct AuthUserRepository {
    inner: Arc<Mutex<InnerRepo>>,
}

struct InnerRepo {
    next_id: u32,
    users_by_username: HashMap<String, AuthUser>,
}

impl AuthUserRepository {
    /// Create a new authentication user repository.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerRepo {
                next_id: 1,
                users_by_username: HashMap::new(),
            })),
        }
    }

    /// Find a user by username
    pub fn find_by_username(&self, username: &str) -> Option<AuthUser> {
        let repo = self.inner.lock();
        repo.users_by_username.get(username).cloned()
    }

    /// Create and store a new user with a unique ID
    pub fn create(&self, username: String, password_hash: String) -> AuthUser {
        let mut repo = self.inner.lock();

        let user = AuthUser {
            id: repo.next_id,
            username: username.clone(),
            password_hash,
        };

        repo.next_id += 1;
        repo.users_by_username.insert(username, user.clone());

        user
    }
}

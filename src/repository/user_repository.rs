use crate::models::user::User;

// In-memory storage for users
pub struct UserRepository {
    users: Vec<User>,
}

impl UserRepository {
    pub fn new() -> Self {
        UserRepository { users: Vec::new() }
    }

    // Add a new user
    pub fn add(&mut self, user: User) {
        self.users.push(user);
    }

    // Find user by ID
    pub fn find_by_id(&self, id: u32) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }

    // Find user by username
    pub fn find_by_username(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|u| u.username == username)
    }

    // Get all active users
    pub fn get_active_users(&self) -> Vec<&User> {
        self.users.iter().filter(|u| u.active).collect()
    }

    // Count total users
    pub fn count(&self) -> usize {
        self.users.len()
    }
}
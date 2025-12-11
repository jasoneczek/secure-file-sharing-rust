pub struct User {
    pub id: u32,
    pub username: String,
    pub password_hash: String,
    pub created_at: i64,
    pub active: bool,
}

impl User {
    pub fn new(id: u32, username: String, password_hash: String) -> User {
        User {
            id,
            username,
            password_hash,
            created_at: 1699564800,
            active: true,
        }
    }

    pub fn display_info(&self) {
        println!("User: {} (ID: {}) - Active: {}", self.username, self.id, self.active);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn update_password(&mut self, new_password_hash: String) {
        self.password_hash = new_password_hash;
    }

    // Check if username meets minimum requirements
    pub fn has_valid_username(&self) -> bool {
        !self.username.is_empty() && self.username.len() >= 3
    }

    // Check if password has is set
    pub fn has_password(&self) -> bool {
        !self.password_hash.is_empty()
    }

    // Return length of username
    pub fn username_length(&self) -> usize {
        self.username.len()
    }
}
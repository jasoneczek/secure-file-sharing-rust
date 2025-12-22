#[derive(Debug)]
pub enum UserError {
    InvalidUsername(String),
    EmptyPassword,
    UsernameTooShort,
}

pub struct User {
    pub id: u32,
    pub username: String,
    pub password_hash: String,
    pub created_at: i64,
    pub active: bool,
    pub email: Option<String>,
}

impl User {
    pub fn new(id: u32, username: String, password_hash: String) -> User {
        User {
            id,
            username,
            password_hash,
            created_at: 1699564800,
            active: true,
            email: None,
        }
    }

    // Display user info
    pub fn display_info(&self) {
        println!(
            "User: {} (ID: {}) - Active: {}",
            self.username, self.id, self.active
        );
        match &self.email {
            Some(email) => println!("Email: {}", email),
            None => println!("Email: Not provided"),
        }
    }

    // Check user status
    pub fn is_active(&self) -> bool {
        self.active
    }

    // Deactivate user
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    // Activate user
    pub fn activate(&mut self) {
        self.active = true;
    }

    // Update password
    pub fn update_password(&mut self, new_password_hash: String) {
        self.password_hash = new_password_hash;
    }

    // Validate username
    pub fn validate_username(&self) -> Result<(), UserError> {
        if self.username.is_empty() {
            return Err(UserError::InvalidUsername(
                "Username cannot be empty".to_string(),
            ));
        }
        if self.username.len() < 3 {
            return Err(UserError::UsernameTooShort);
        }
        Ok(())
    }

    // Validate password
    pub fn validate_password(&self) -> Result<(), UserError> {
        if self.password_hash.is_empty() {
            return Err(UserError::EmptyPassword);
        }
        Ok(())
    }

    // Return length of username
    pub fn username_length(&self) -> usize {
        self.username.len()
    }

    // Set email
    pub fn set_email(&mut self, email: String) {
        self.email = Some(email);
    }

    // Get email
    pub fn get_email(&self) -> Option<&String> {
        self.email.as_ref()
    }
}

impl crate::traits::Identifiable for User {
    fn id(&self) -> u32 {
        self.id
    }
}

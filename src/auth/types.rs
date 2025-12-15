use serde::{Deserialize, Serialize};

/// Request body for `POST /register`.
///
/// Contains the user's chosen username and plaintext password.
/// The password will be hashed before storage.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// Request body for `POST /login`.
///
/// Contains the user's username and plaintext password.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response body returned after successful authentication.
///
/// The access token is used for authenticated requests.
/// `expires_in` represents the token lifetime in seconds.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

/// Internal representation of an authenticated user
///
/// This type is used by the authentication service and repository.
/// It is not exposed over the HTTP API
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: u32,
    pub username: String,
    pub password_hash: String,
}
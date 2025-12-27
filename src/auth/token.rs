use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// sub = user id
/// exp = expiration timestamp
/// iat = issued-at timestamp
/// jti = unique token id

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u32,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

/// Secret key to move to env config later
const SECRET: &[u8] = b"secret-key-to-be-changed";

const ACCESS_TOKEN_TTL_SECS: u64 = 3600;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs()
}

/// Create a JWT for a given user id
pub fn create_token(user_id: u32) -> Result<String, jsonwebtoken::errors::Error> {
    let now = now_secs();
    let expiration = now + ACCESS_TOKEN_TTL_SECS;

    let claims = Claims {
        sub: user_id,
        exp: expiration as usize,
        iat: now as usize,
        jti: Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
}

/// Verify a JWT and return its claims
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::default(),
    )?;

    Ok(data.claims)
}

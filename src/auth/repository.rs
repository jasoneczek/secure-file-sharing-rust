use sqlx::{Row, SqlitePool};

use crate::auth::types::AuthUser;

/// DB backed repository for authentication users
#[derive(Clone)]
pub struct AuthUserRepository {
    pool: SqlitePool,
}

impl AuthUserRepository {
    /// Create a new authentication user repository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find a user by username
    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<AuthUser>, sqlx::Error> {
        let row_opt = sqlx::query(
            r#"
            SELECT id, username, password_hash
            FROM users
            WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row_opt.map(|row| AuthUser {
            id: row.get::<i64, _>("id") as u32,
            username: row.get::<String, _>("username"),
            password_hash: row.get::<String, _>("password_hash"),
        }))
    }

    /// Create and store a new user with a unique ID
    pub async fn create(
        &self,
        username: String,
        password_hash: String,
    ) -> Result<AuthUser, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, active)
            VALUES (?, ?, 1)
            "#,
        )
        .bind(&username)
        .bind(&password_hash)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid() as u32;

        Ok(AuthUser {
            id,
            username,
            password_hash,
        })
    }
}

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
    pub async fn find_by_username(&self, username: &str) -> Result<Option<AuthUser>, sqlx::Error> {
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

    /// Get username by user id
    pub async fn get_username_by_id(&self, user_id: u32) -> Result<Option<String>, sqlx::Error> {
        let row_opt = sqlx::query(
            r#"
            SELECT username
            FROM users
            WHERE id = ?1
            "#,
        )
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row_opt.map(|row| row.get::<String, _>("username")))
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

    /// Store a refresh token for a user
    pub async fn insert_refresh_token(&self, user_id: u32, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token)
            VALUES (?1, ?2)
            "#,
        )
        .bind(user_id as i64)
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Rotates a refresh token in a single transaction: the old token must exist and be unrevoked,
    /// it is marked revoked and linked via `replaced_by`, and a new token row is inserted.
    /// Returns `Some(user_id)` on success, or `None` if the token is invalid or already used.
    pub async fn rotate_refresh_token(
        &self,
        old_token: &str,
        new_token: &str,
    ) -> Result<Option<u32>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Look up old token
        let row_opt = sqlx::query(
            r#"
            SELECT user_id, revoked_at
            FROM refresh_tokens
            WHERE token = ?1
            "#,
        )
        .bind(old_token)
        .fetch_optional(&mut *tx)
        .await?;

        let row = match row_opt {
            Some(r) => r,
            None => {
                tx.rollback().await?;
                return Ok(None);
            }
        };

        let revoked_at: Option<i64> = row.get("revoked_at");
        if revoked_at.is_some() {
            tx.rollback().await?;
            return Ok(None);
        }

        let user_id: i64 = row.get("user_id");

        // Revoke old token
        let upd = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = strftime('%s', 'now'),
                replaced_by = ?1
            WHERE token = ?2
                AND revoked_at IS NULL
            "#,
        )
        .bind(new_token)
        .bind(old_token)
        .execute(&mut *tx)
        .await?;

        if upd.rows_affected() != 1 {
            tx.rollback().await?;
            return Ok(None);
        }

        // Insert new token
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token)
            VALUES (?1, ?2)
            "#,
        )
        .bind(user_id)
        .bind(new_token)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(Some(user_id as u32))
    }

    // Revoke all refresh tokens
    pub async fn revoke_all_refresh_tokens_for_user(
        &self,
        user_id: u32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = strftime('%s','now')
            WHERE user_id = ?1 AND revoked_at IS NULL
            "#,
        )
        .bind(user_id as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

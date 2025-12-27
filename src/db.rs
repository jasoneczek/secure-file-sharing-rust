use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::PathBuf;
use tokio::fs;

/// Initialize SQLite database and create tables if missing
/// DB file will be created in data/app.db
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // ensure ./data exists
    let _ = fs::create_dir_all("data").await;

    let db_path: PathBuf = std::env::current_dir()
        .expect("current_dir failed")
        .join("data")
        .join("app.db");

    println!("Using db file: {}", db_path.display());

    let opts = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .pragma("foreign_keys", "ON");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            active INTEGER NOT NULL DEFAULT 1,
            email TEXT
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            size INTEGER NOT NULL,
            owner_id INTEGER NOT NULL,
            is_public INTEGER NOT NULL DEFAULT 0,
            uploaded_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            description TEXT,
            FOREIGN KEY(owner_id) REFERENCES users(id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS permissions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            permission_type TEXT NOT NULL,
            FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE(file_id, user_id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            token TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            revoked_at INTEGER,
            replaced_by TEXT,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id
        ON refresh_tokens(user_id);
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

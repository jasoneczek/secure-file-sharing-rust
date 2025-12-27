use sqlx::SqlitePool;

pub mod auth;
pub mod auth_middleware;
pub mod file;
pub mod health;
pub mod me;

pub use health::health_check;

use crate::auth::service::SimpleAuthService;

#[derive(Clone)]
pub struct AppState {
    pub auth: SimpleAuthService,
    pub db: SqlitePool,
}

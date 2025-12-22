use parking_lot::Mutex;
use sqlx::SqlitePool;
use std::sync::Arc;

pub mod auth;
pub mod auth_middleware;
pub mod file;
pub mod health;
pub mod me;

pub use health::health_check;

use crate::auth::service::SimpleAuthService;
use crate::repository::{FileRepository, PermissionRepository, UserRepository};

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<Mutex<UserRepository>>,
    pub files: Arc<Mutex<FileRepository>>,
    pub permissions: Arc<Mutex<PermissionRepository>>,
    pub auth: SimpleAuthService,
    pub db: SqlitePool,
}

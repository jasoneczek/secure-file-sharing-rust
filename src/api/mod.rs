use std::sync::Arc;
use parking_lot::Mutex;

pub mod health;
pub use health::health_check;

use crate::repository::{
    UserRepository,
    FileRepository,
    PermissionRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<Mutex<UserRepository>>,
    pub files: Arc<Mutex<FileRepository>>,
    pub permissions: Arc<Mutex<PermissionRepository>>,
}
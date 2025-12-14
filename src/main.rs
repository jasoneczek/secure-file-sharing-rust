mod models;
mod traits;
mod repository;
mod api;
mod auth;

use std::sync::Arc;
use parking_lot::Mutex;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

use api::{AppState, health_check};
use repository::{UserRepository, FileRepository, PermissionRepository};

#[tokio::main]
async fn main() {
    println!("\n=== File Sharing Server ===");

    // Build application state
    let state = AppState {
        users: Arc::new(Mutex::new(UserRepository::new())),
        files: Arc::new(Mutex::new(FileRepository::new())),
        permissions: Arc::new(Mutex::new(PermissionRepository::new())),
    };

    // Build Axum router
    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(state);

    // Listener
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running at http://0.0.0.0:8080/");

    // Start server
    axum::serve(listener, app)
        .await
        .unwrap();
}
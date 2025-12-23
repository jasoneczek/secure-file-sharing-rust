mod api;
mod auth;
mod db;
mod file;
mod models;
mod repository;
mod storage;
mod traits;

use parking_lot::Mutex;
use std::sync::Arc;

use axum::routing::delete;
use axum::{
    Router, middleware,
    routing::{get, post},
};

use tokio::net::TcpListener;

use api::auth::{login_handler, refresh_handler, register_handler};
use api::auth_middleware::auth_middleware;
use api::file::{
    download_handler, download_public_handler, revoke_share_by_user_handler, revoke_share_handler,
    share_file_handler, upload_handler,
};
use api::me::me_handler;
use api::{AppState, health_check};

use auth::repository::AuthUserRepository;
use auth::service::SimpleAuthService;
use repository::{FileRepository, PermissionRepository, UserRepository};

#[tokio::main]
async fn main() {
    println!("\n=== File Sharing Server ===");

    // Initialize SQLite DB
    let db_pool = db::init_db().await.expect("DB init failed");

    // Build auth service
    let auth_repo = AuthUserRepository::new(db_pool.clone());
    let auth_service = SimpleAuthService::new(auth_repo);

    // Build application state
    let state = AppState {
        users: Arc::new(Mutex::new(UserRepository::new())),
        files: Arc::new(Mutex::new(FileRepository::new())),
        permissions: Arc::new(Mutex::new(PermissionRepository::new())),
        auth: auth_service,
        db: db_pool,
    };

    // Public routes
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/token/refresh", get(refresh_handler))
        .route("/file/public/:id", get(download_public_handler));

    // Protected routes
    let protected_routes = Router::new()
        .route("/me", get(me_handler))
        .route("/file/upload", post(upload_handler))
        .route("/file/:id", get(download_handler))
        .route("/file/:id/share", post(share_file_handler))
        .route(
            "/file/:id/share/:permission_id",
            delete(revoke_share_handler),
        )
        .route(
            "/file/:id/share/user/:user_id",
            delete(revoke_share_by_user_handler),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Combine routers
    let app = public_routes.merge(protected_routes).with_state(state);

    // Listener
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running at http://0.0.0.0:8080/");

    // Start server
    axum::serve(listener, app).await.unwrap();
}

use axum::{Json, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub message: &'static str,
}

pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        message: "File Sharing Server is running",
    })
}
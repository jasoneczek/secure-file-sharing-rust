use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use serde::Serialize;
use sqlx::Row;

use crate::api::AppState;

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: u32,
    pub username: String,
}

pub async fn me_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<u32>,
) -> Result<Json<MeResponse>, StatusCode> {
    let row = sqlx::query("SELECT username FROM users WHERE id = ?1")
        .bind(user_id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::UNAUTHORIZED)?;
    let username: String = row.get("username");

    Ok(Json(MeResponse { user_id, username }))
}

use axum::{Json, extract::Extension};
use serde::Serialize;

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: u32,
}

pub async fn me_handler(Extension(user_id): Extension<u32>) -> Json<MeResponse> {
    Json(MeResponse { user_id })
}

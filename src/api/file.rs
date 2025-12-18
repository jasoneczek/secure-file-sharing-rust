use axum::{
    extract::{Multipart, State, Extension, Path},
    http::{StatusCode, header},
    response::Response,
    body::Body,
    Json,
};
use serde::Serialize;
use bytes::Bytes;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;

use crate::api::AppState;
use crate::models::file::File;
use crate::storage::disk::{
    ensure_upload_dir,
    temp_upload_path,
    final_upload_path,
    write_file_atomic,
};

/// Maximum allowed upload size 10 MB
const MAX_UPLOAD_SIZE: u64 = 10 * 1024 * 1024;

#[derive(Serialize)]
pub struct UploadResponse {
    pub file_id: u32,
    pub filename: String,
    pub size: u64,
}

/// Handle authenticated file uploads
#[axum::debug_handler]
pub async fn upload_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<u32>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    let mut file_chunks: Vec<Bytes> = Vec::new();
    let mut filename: Option<String> = None;

    // Read multipart fields
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if field.name() == Some("file") {
            filename = field.file_name().map(|s| s.to_string());

            let data = field
                .bytes()
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            file_chunks.push(data);
        }
    }

    let filename = filename.ok_or(StatusCode::BAD_REQUEST)?;

    // Ensure upload directory exists
    ensure_upload_dir()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create file ID
    let file_id = {
        let files = state.files.lock();
        (files.count() + 1) as u32
    };

    let temp = temp_upload_path();
    let final_path = final_upload_path(file_id as u64);

    // Write file to disk atomically
    let size = write_file_atomic(
        &temp,
        &final_path,
        &file_chunks,
        MAX_UPLOAD_SIZE,
    )
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Store file metadata
    {
        let mut files = state.files.lock();
        let file = File::new(
                file_id,
                filename.clone(),
                size,
                user_id,
            );
        files.add(file);
    }

    Ok(Json(UploadResponse {
        file_id,
        filename,
        size,
    }))
}

/// Handle authenticated file downloads
pub async fn download_handler(
    Path(file_id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let file = {
        let files = state.files.lock();
        files.find_by_id(file_id).cloned()
    }
    .ok_or(StatusCode::NOT_FOUND)?;

    let path = final_upload_path(file.id as u64);

    let mut disk_file = TokioFile::open(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut buffer = Vec::new();
    disk_file
        .read_to_end(&mut buffer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut response = Response::new(buffer.into());
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/octet-stream"),
    );

    Ok(response)
}
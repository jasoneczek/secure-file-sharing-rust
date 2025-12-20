use axum::{
    extract::{Multipart, State, Extension, Path},
    http::{StatusCode, header, HeaderValue},
    response::Response,
    body::Body,
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::api::AppState;
use crate::file::service::FileService;
use crate::models::file::File;
use crate::models::permission::{Permission, PermissionType};
use crate::storage::disk::{
    ensure_upload_dir,
    temp_upload_path,
    final_upload_path,
};

/// Maximum allowed upload size 10 MB
const MAX_UPLOAD_SIZE: u64 = 10 * 1024 * 1024;

#[derive(Serialize)]
pub struct UploadResponse {
    pub file_id: u32,
    pub filename: String,
    pub size: u64,
    pub is_public: bool,
}

#[derive(Deserialize)]
pub struct ShareRequest {
    pub user_id: u32,
}

#[derive(Serialize)]
pub struct ShareResponse {
    pub permission_id: u32,
    pub file_id: u32,
    pub user_id: u32,
}

/// Handle authenticated file uploads
#[axum::debug_handler]
pub async fn upload_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<u32>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    ensure_upload_dir()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Allocate file ID
    let file_id = {
        let files = state.files.lock();
        (files.count() + 1) as u32
    };

    let temp_path = temp_upload_path();
    let final_path = final_upload_path(file_id as u64);

    let mut original_filename: Option<String> = None;
    let mut wrote_file = false;
    let mut size: u64 = 0;
    let mut is_public: bool = false;

    // Create temp file
    let mut temp_file: Option<TokioFile> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?

    {
        match field.name() {
            Some("is_public") => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                is_public = matches!(
                    text.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                );
            }

            Some("file") => {
                original_filename = field.file_name().map(|s| s.to_string());

                let file = TokioFile::create(&temp_path)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                temp_file = Some(file);

                let mut field = field;
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?
                {
                    size = size
                        .checked_add(chunk.len() as u64)
                        .ok_or(StatusCode::PAYLOAD_TOO_LARGE)?;
                    if size > MAX_UPLOAD_SIZE {
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        return Err(StatusCode::PAYLOAD_TOO_LARGE);
                    }

                    if let Some(f) = temp_file.as_mut() {
                        f.write_all(&chunk)
                            .await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    }
                }

                wrote_file = true;
            }

            _ => {
                // Ignore unknown fields for now
            }
        }
    }

    if !wrote_file {
        return Err(StatusCode::BAD_REQUEST);
    }
    let filename = original_filename.ok_or(StatusCode::BAD_REQUEST)?;

    // Close file before rename
    drop(temp_file);

    tokio::fs::rename(&temp_path, &final_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Store metadata
    {
        let mut files = state.files.lock();
        let file = File::new(file_id, filename.clone(), size, user_id, is_public);
        files.add(file);
    }

    Ok(Json(UploadResponse {
        file_id,
        filename,
        size,
        is_public,
    }))
}

/// Handle authenticated file downloads
pub async fn download_handler(
    Path(file_id): Path<u32>,
    State(state): State<AppState>,
    Extension(user_id): Extension<u32>,
) -> Result<Response, StatusCode> {
    let (stored_id, filename_for_header) = {
        let files = state.files.lock();
        let perms = state.permissions.lock();
        let svc = FileService::new(&*files, &*perms);

        let file = svc
            .get_for_download(user_id, file_id)
            .ok_or(StatusCode::NOT_FOUND)?;

        (file.id, file.filename.clone())
    };

    stream_file_response(stored_id, filename_for_header).await
}

/// Public download (no auth): only works if file.is_public == true
pub async fn download_public_handler(
    Path(file_id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let (stored_id, filename_for_header) = {
        let files = state.files.lock();
        let perms = state.permissions.lock();
        let svc = FileService::new(&*files, &*perms);

        let file = svc
            .get_public_for_download(file_id)
            .ok_or(StatusCode::NOT_FOUND)?;

        (file.id, file.filename.clone())
    };

    stream_file_response(stored_id, filename_for_header).await
}

/// Owner-only: grant another user access to a file
pub async fn share_file_handler(
    Path(file_id): Path<u32>,
    State(state): State<AppState>,
    Extension(owner_id): Extension<u32>,
    Json(req): Json<ShareRequest>,
) -> Result<Json<ShareResponse>, StatusCode> {
    // File exists and caller is owner
    {
        let files = state.files.lock();
        let file = files.find_by_id(file_id).ok_or(StatusCode::NOT_FOUND)?;
        if file.owner_id != owner_id {
            return Err(StatusCode::NOT_FOUND); // anti-leak
        }
    }

    // Prevent duplicate share
    {
        let perms = state.permissions.lock();
        if perms.user_has_access(req.user_id, file_id) {
            return Err(StatusCode::CONFLICT);
        }
    }

    // Create id and add permission
    let permission_id = {
        let perms = state.permissions.lock();
        (perms.count() + 1) as u32
    };

    {
        let mut perms = state.permissions.lock();
        perms.add(Permission::new(
            permission_id,
            file_id,
            req.user_id,
            PermissionType::Shared,
        ));
    }

    Ok(Json(ShareResponse {
        permission_id,
        file_id,
        user_id: req.user_id,
    }))
}

/// Owner-only: revoke a specific permission by id
pub async fn revoke_share_handler(
    Path((file_id, permission_id)): Path<(u32, u32)>,
    State(state): State<AppState>,
    Extension(owner_id): Extension<u32>,
) -> Result<StatusCode, StatusCode> {
    // File exists and caller owns it
    {
        let files = state.files.lock();
        let file = files.find_by_id(file_id).ok_or(StatusCode::NOT_FOUND)?;
        if file.owner_id != owner_id {
            return Err(StatusCode::NOT_FOUND); // anti-leak
        }
    }

    // Permission exists and belongs to this file
    {
        let perms = state.permissions.lock();
        let p = perms.find_by_id(permission_id).ok_or(StatusCode::NOT_FOUND)?;
        if p.file_id != file_id {
            return Err(StatusCode::NOT_FOUND);
        }
    }

    // Remove
    {
        let mut perms = state.permissions.lock();
        perms.remove(permission_id);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Stream file with headers
async fn stream_file_response(file_id: u32, filename_for_header: String) -> Result<Response, StatusCode> {
    let path = final_upload_path(file_id as u64);

    let disk_file = TokioFile::open(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let stream = ReaderStream::new(disk_file);
    let body = Body::from_stream(stream);

    let mut response = Response::new(body);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    let safe_name = filename_for_header.replace('"', "_");
    let disposition = format!("attachment; filename=\"{}\"", safe_name);
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        disposition
            .parse()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    Ok(response)
}
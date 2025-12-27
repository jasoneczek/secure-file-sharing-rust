use axum::{
    Json,
    body::Body,
    extract::{Extension, Multipart, Path, State},
    http::{HeaderValue, StatusCode, header},
    response::Response,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::api::AppState;
use crate::storage::disk::{ensure_upload_dir, final_upload_path, temp_upload_path};

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

    let temp_path = temp_upload_path();

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
                while let Some(chunk) = field.chunk().await.map_err(|_| StatusCode::BAD_REQUEST)? {
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

    // Insert into DB first to get a stable file_id
    let uploaded_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .as_secs() as i64;

    let res = sqlx::query(
        r#"
        INSERT INTO files (filename, size, owner_id, is_public, uploaded_at, description)
        VALUES (?1, ?2, ?3, ?4, ?5, NULL)
        "#,
    )
    .bind(&filename)
    .bind(size as i64)
    .bind(user_id as i64)
    .bind(if is_public { 1i64 } else { 0i64 })
    .bind(uploaded_at)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file_id = res.last_insert_rowid() as u32;

    // Move temp file into final path using file_id
    let final_path = final_upload_path(file_id as u64);
    if tokio::fs::rename(&temp_path, &final_path).await.is_err() {
        let _ = sqlx::query("DELETE FROM files WHERE id = ?1")
            .bind(file_id as i64)
            .execute(&state.db)
            .await;
        let _ = tokio::fs::remove_file(&temp_path).await;

        return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
    let row = sqlx::query(
        r#"
        SELECT f.id, f.filename
        FROM files f
        WHERE f.id = ?1
            AND (
                f.owner_id = ?2
                OR EXISTS (
                    SELECT 1
                    FROM permissions p
                    WHERE p.file_id = f.id
                        AND p.user_id = ?2
                )
            )
        "#,
    )
    .bind(file_id as i64)
    .bind(user_id as i64)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::NOT_FOUND)?;
    let stored_id: i64 = row.get("id");
    let filename_for_header: String = row.get("filename");

    stream_file_response(stored_id as u32, filename_for_header).await
}

/// Public download (no auth): only works if file.is_public == 1
pub async fn download_public_handler(
    Path(file_id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT id, filename
        FROM files
        WHERE id = ?1 AND is_public = 1
        "#,
    )
    .bind(file_id as i64)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::NOT_FOUND)?;
    let stored_id: i64 = row.get("id");
    let filename_for_header: String = row.get("filename");

    stream_file_response(stored_id as u32, filename_for_header).await
}

/// Owner-only: grant another user access to a file
pub async fn share_file_handler(
    Path(file_id): Path<u32>,
    State(state): State<AppState>,
    Extension(owner_id): Extension<u32>,
    Json(req): Json<ShareRequest>,
) -> Result<Json<ShareResponse>, StatusCode> {
    // Verify file exists and caller is owner
    let row = sqlx::query("SELECT owner_id FROM files WHERE id = ?1")
        .bind(file_id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::NOT_FOUND)?;
    let db_owner_id: i64 = row.get("owner_id");
    if db_owner_id != owner_id as i64 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Return 404 if target user doesn't exist
    let target_exists = sqlx::query("SELECT 1 FROM users WHERE id = ?1 LIMIT 1")
        .bind(req.user_id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if target_exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Insert permission
    let res = sqlx::query(
        r#"
        INSERT INTO permissions (file_id, user_id, permission_type)
        VALUES (?1, ?2, 'Shared')
        "#,
    )
    .bind(file_id as i64)
    .bind(req.user_id as i64)
    .execute(&state.db)
    .await;

    let res = match res {
        Ok(r) => r,
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                let msg = db_err.message().to_ascii_lowercase();
                if msg.contains("unique") || msg.contains("constraint") {
                    return Err(StatusCode::CONFLICT);
                }
            }
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let permission_id = res.last_insert_rowid() as u32;

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
    // Verify if file exists and caller owns it
    let row = sqlx::query("SELECT owner_id FROM files WHERE id = ?1")
        .bind(file_id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let db_owner_id: i64 = row.get("owner_id");
    if db_owner_id != owner_id as i64 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Verify permission belongs to this file
    let prow = sqlx::query("SELECT file_id FROM permissions WHERE id = ?1")
        .bind(permission_id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let p_file_id: i64 = prow.get("file_id");
    if p_file_id != file_id as i64 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Delete
    sqlx::query("DELETE FROM permissions WHERE id = ?1")
        .bind(permission_id as i64)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// Revoke a share by target user id
pub async fn revoke_share_by_user_handler(
    Path((file_id, target_user_id)): Path<(u32, u32)>,
    State(state): State<AppState>,
    Extension(owner_id): Extension<u32>,
) -> Result<StatusCode, StatusCode> {
    // Verify file exists and caller owns it
    let row = sqlx::query("SELECT owner_id FROM files WHERE id = ?1")
        .bind(file_id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let db_owner_id: i64 = row.get("owner_id");
    if db_owner_id != owner_id as i64 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Find the permission id for (file_id, target_user_id)
    let row = sqlx::query("SELECT id FROM permissions WHERE file_id = ?1 AND user_id = ?2 LIMIT 1")
        .bind(file_id as i64)
        .bind(target_user_id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let permission_id: i64 = row.get("id");

    // Delete
    sqlx::query("DELETE FROM permissions WHERE id = ?1")
        .bind(permission_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Stream file with headers
async fn stream_file_response(
    file_id: u32,
    filename_for_header: String,
) -> Result<Response, StatusCode> {
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

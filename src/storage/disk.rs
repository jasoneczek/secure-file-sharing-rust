use std::path::{Path, PathBuf};

use tokio::fs;
use uuid::Uuid;

/// Base directory for all uploaded files
const UPLOAD_DIR: &str = "data/uploads";

/// Ensure the upload directory exists on disk
pub async fn ensure_upload_dir() -> std::io::Result<()> {
    fs::create_dir_all(UPLOAD_DIR).await
}

/// Generate a temporary upload path
pub fn temp_upload_path() -> PathBuf {
    let filename = format!("tmp_{}", Uuid::new_v4());
    Path::new(UPLOAD_DIR).join(filename)
}

/// Generate a final storage path
pub fn final_upload_path(file_id: u64) -> PathBuf {
    let filename = format!("{}.bin", file_id);
    Path::new(UPLOAD_DIR).join(filename)
}

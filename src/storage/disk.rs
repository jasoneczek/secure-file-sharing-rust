use std::path::{Path, PathBuf};

use bytes::Bytes;
use tokio::fs;
use tokio::io::AsyncWriteExt;
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

/// Write uploaded bytes safely to disk using a temp file
pub async fn write_file_atomic(
    temp_path: &Path,
    final_path: &Path,
    chunks: &[Bytes],
    max_size: u64,
) -> std::io::Result<u64> {
    let mut file = fs::File::create(temp_path).await?;
    let mut written: u64 = 0;

    for chunk in chunks {
        written += chunk.len() as u64;

        if written > max_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File too large",
            ));
        }

        file.write_all(&chunk).await?;
    }

    file.flush().await?;
    drop(file);

    fs::rename(temp_path, final_path).await?;

    Ok(written)
}

/// Read a file fully from disk
pub async fn read_file(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    fs::read(path).await
}

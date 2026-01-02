use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AuthReq<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Deserialize)]
pub struct AuthResp {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct TokenStore {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct ShareResp {
    pub permission_id: u32,
    pub file_id: u32,
    pub user_id: u32,
}

#[derive(Deserialize)]
pub struct UploadResp {
    pub file_id: u32,
    pub filename: String,
    pub size: u64,
    pub is_public: bool,
}

#[derive(Debug, Deserialize)]
pub struct FileListItem {
    pub file_id: u32,
    pub filename: String,
    pub size: u64,
    pub is_public: bool,
    pub uploaded_at: i64,
    pub access: String,
}

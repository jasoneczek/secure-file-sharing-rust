use crate::types::TokenStore;
use std::{fs, path::PathBuf};
use directories::ProjectDirs;

pub fn token_path() -> PathBuf {
    let proj = ProjectDirs::from("com", "programming-3", "sfs")
        .expect("Could not determine a config directory");
    proj.config_dir().join("tokens.json")
}

pub fn save_tokens(store: &TokenStore) -> std::io::Result<()> {
    let p = token_path();
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    let s = serde_json::to_string_pretty(store).unwrap();
    fs::write(p, s)
}

pub fn load_tokens() -> std::io::Result<TokenStore> {
    let p = token_path();
    let s = fs::read_to_string(p)?;
    Ok(serde_json::from_str(&s).unwrap_or_default())
}

pub fn require_access(store: &TokenStore) -> Option<&str> {
    if store.access_token.is_empty() {
        None
    } else {
        Some(store.access_token.as_str())
    }
}

pub fn require_refresh(store: &TokenStore) -> Option<&str> {
    if store.refresh_token.is_empty() {
        None
    } else {
        Some(store.refresh_token.as_str())
    }
}

pub fn logout_local() -> Result<(), std::io::Error> {
    let p = token_path();
    match std::fs::remove_file(&p) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
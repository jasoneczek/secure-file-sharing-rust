use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "sfs", about = "Secure File Sharing CLI client")]
struct Cli {
    #[arg(long, default_value = "http://localhost:8080")]
    base: String,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    Health,
    Register { username: String, password: String },
    Login { username: String, password: String },
    Me,
    Refresh,
    Upload {
        path: String,
        #[arg(long, default_value_t = false)]
        public: bool,
    },
    Download {
        file_id: u32,
        #[arg(long)]
        out: String,
    },
    Share {
        file_id: u32,
        user_id: u32,
    },
    RevokeUser {
        file_id: u32,
        user_id: u32,
    },
    PublicDownload {
        file_id: u32,
        #[arg(long)]
        out: String,
    },
    Logout,
}

#[derive(Serialize)]
struct AuthReq<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct AuthResp {
    access_token: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Default)]
struct TokenStore {
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct ShareResp {
    permission_id: u32,
    file_id: u32,
    user_id: u32,
}

fn token_path() -> PathBuf {
    PathBuf::from("client").join(".sfs").join("tokens.json")
}

fn save_tokens(store: &TokenStore) -> std::io::Result<()> {
    let p = token_path();
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    let s = serde_json::to_string_pretty(store).unwrap();
    fs::write(p, s)
}

fn load_tokens() -> std::io::Result<TokenStore> {
    let p = token_path();
    let s = fs::read_to_string(p)?;
    Ok(serde_json::from_str(&s).unwrap_or_default())
}

fn require_access(store: &TokenStore) -> Option<&str> {
    if store.access_token.is_empty() {
        None
    } else {
        Some(store.access_token.as_str())
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Health => {
            let url = format!("{}/health", cli.base);

            let resp = match reqwest::get(&url).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to connect to {url}: {e}");
                    return;
                }
            };

            if !resp.status().is_success() {
                eprintln!("Health check failed: HTTP {}", resp.status());
                return;
            }

            if let Err(e) = resp.text().await {
                eprintln!("Failed to read response body: {e}");
                return;
            }

            println!("Health OK");
        }

        Command::Register { username, password } => {
            let url = format!("{}/register", cli.base);

            let resp = reqwest::Client::new()
                .post(url)
                .json(&AuthReq {
                    username: &username,
                    password: &password,
                })
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Register request failed: {e}");
                    return;
                }
            };

            if !resp.status().is_success() {
                eprintln!("Register failed: HTTP {}", resp.status());
                let _ = resp.text().await;
                return;
            }

            let auth: AuthResp = match resp.json().await {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Failed to parse JSON: {e}");
                    return;
                }
            };

            let store = TokenStore {
                access_token: auth.access_token,
                refresh_token: auth.refresh_token,
            };

            if let Err(e) = save_tokens(&store) {
                eprintln!("Failed to save tokens: {e}");
                return;
            }

            println!("Registered. Tokens saved to {:?}", token_path());
        }
        Command::Login { username, password } => {
            let url = format!("{}/login", cli.base);

            let resp = reqwest::Client::new()
                .post(url)
                .json(&AuthReq {
                    username: &username,
                    password: &password,
                })
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Login request failed: {e}");
                    return;
                }
            };

            if !resp.status().is_success() {
                eprintln!("Login failed: HTTP {}", resp.status());
                let _ = resp.text().await;
                return;
            }

            let auth: AuthResp = match resp.json().await {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Failed to parse JSON: {e}");
                    return;
                }
            };

            let store = TokenStore {
                access_token: auth.access_token,
                refresh_token: auth.refresh_token,
            };

            if let Err(e) = save_tokens(&store) {
                eprintln!("Failed to save tokens: {e}");
                return;
            }

            println!("Logged in. Tokens saved to {:?}", token_path());
        }
        Command::Me => {
            let store = match load_tokens() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("No saved tokens. Run: client login <user> <pass>");
                    return;
                }
            };

            let tok = match require_access(&store) {
                Some(t) => t,
                None => {
                    eprintln!("No access token saved. Run: client login <user> <pass>");
                    return;
                }
            };

            let url = format!("{}/me", cli.base);
            let resp = reqwest::Client::new()
                .get(url)
                .bearer_auth(tok)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Request failed: {e}");
                    return;
                }
            };

            println!("HTTP {}", resp.status());
        }

        Command::Refresh => {
            let mut store = match load_tokens() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("No saved tokens. Run: client login <user> <pass>");
                    return;
                }
            };

            if store.refresh_token.is_empty() {
                eprintln!("No refresh token saved. Login again.");
                return;
            }

            let url = format!("{}/token/refresh", cli.base);

            let resp = reqwest::Client::new()
                .get(url)
                .bearer_auth(&store.refresh_token) // refresh token goes in Authorization: Bearer ...
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Refresh request failed: {e}");
                    return;
                }
            };

            if !resp.status().is_success() {
                eprintln!("Refresh failed: HTTP {}", resp.status());
                let _ = resp.text().await;
                return;
            }

            let auth: AuthResp = match resp.json().await {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Failed to parse JSON: {e}");
                    return;
                }
            };

            store.access_token = auth.access_token;
            store.refresh_token = auth.refresh_token;

            if let Err(e) = save_tokens(&store) {
                eprintln!("Failed to save tokens: {e}");
                return;
            }

            println!("Refreshed. Tokens updated at {:?}", token_path());
        }

        Command::Upload { path, public } => {
            // Load saved tokens
            let store = match load_tokens() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("No saved tokens. Run: client login <user> <pass>");
                    return;
                }
            };

            let tok = match require_access(&store) {
                Some(t) => t,
                None => {
                    eprintln!("No access token saved. Run: client login <user> <pass>");
                    return;
                }
            };

            // Read file from disk
            let bytes = match tokio::fs::read(&path).await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read file {path}: {e}");
                    return;
                }
            };

            let file_name = std::path::Path::new(&path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("upload.bin")
                .to_string();

            // Build multipart form: file + is_public
            let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);

            let form = reqwest::multipart::Form::new()
                .part("file", part)
                .text("is_public", public.to_string());

            // Send
            let url = format!("{}/file/upload", cli.base);
            let resp = reqwest::Client::new()
                .post(url)
                .bearer_auth(tok)
                .multipart(form)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Upload request failed: {e}");
                    return;
                }
            };

            println!("HTTP {}", resp.status());
            let body = resp.text().await.unwrap_or_default();
            println!("{body}");
        }
        Command::Download { file_id, out } => {
            let store = match load_tokens() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("No saved tokens. Run: client login <user> <pass>");
                    return;
                }
            };

            let tok = match require_access(&store) {
                Some(t) => t,
                None => {
                    eprintln!("No access token saved. Run: client login <user> <pass>");
                    return;
                }
            };

            let url = format!("{}/file/{}", cli.base, file_id);

            let resp = reqwest::Client::new()
                .get(url)
                .bearer_auth(tok)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Download request failed: {e}");
                    return;
                }
            };

            if !resp.status().is_success() {
                eprintln!("Download failed: HTTP {}", resp.status());
                let _ = resp.text().await;
                return;
            }

            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed reading response bytes: {e}");
                    return;
                }
            };

            if let Err(e) = tokio::fs::write(&out, &bytes).await {
                eprintln!("Failed writing to {out}: {e}");
                return;
            }

            println!("Saved to {out}");
        }
        Command::Share { file_id, user_id } => {
            let store = match load_tokens() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("No saved tokens. Run: client login <user> <pass>");
                    return;
                }
            };

            let tok = match require_access(&store) {
                Some(t) => t,
                None => {
                    eprintln!("No access token saved. Run: client login <user> <pass>");
                    return;
                }
            };

            let url = format!("{}/file/{}/share", cli.base, file_id);

            let resp = reqwest::Client::new()
                .post(url)
                .bearer_auth(tok)
                .json(&serde_json::json!({ "user_id": user_id }))
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Share request failed: {e}");
                    return;
                }
            };

            if !resp.status().is_success() {
                eprintln!("Share failed: HTTP {}", resp.status());
                let _ = resp.text().await;
                return;
            }

            let out: ShareResp = match resp.json().await {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Failed to parse JSON: {e}");
                    return;
                }
            };

            println!(
                "Shared file {} with user {} (permission_id={})",
                out.file_id, out.user_id, out.permission_id
            );
        }
        Command::RevokeUser { file_id, user_id } => {
            let store = match load_tokens() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("No saved tokens. Run: client login <user> <pass>");
                    return;
                }
            };

            let tok = match require_access(&store) {
                Some(t) => t,
                None => {
                    eprintln!("No access token saved. Run: client login <user> <pass>");
                    return;
                }
            };

            let url = format!("{}/file/{}/share/user/{}", cli.base, file_id, user_id);

            let resp = reqwest::Client::new()
                .delete(url)
                .bearer_auth(tok)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Revoke request failed: {e}");
                    return;
                }
            };

            if resp.status() == reqwest::StatusCode::NO_CONTENT {
                println!("Revoked share for user {user_id} on file {file_id}");
                return;
            }

            eprintln!("Revoke failed: HTTP {}", resp.status());
            let _ = resp.text().await;
        }

        Command::PublicDownload { file_id, out } => {
            let url = format!("{}/file/public/{}", cli.base, file_id);

            let resp = reqwest::Client::new().get(url).send().await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Public download request failed: {e}");
                    return;
                }
            };

            if !resp.status().is_success() {
                eprintln!("Public download failed: HTTP {}", resp.status());
                let _ = resp.text().await;
                return;
            }

            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed reading response bytes: {e}");
                    return;
                }
            };

            if let Err(e) = tokio::fs::write(&out, &bytes).await {
                eprintln!("Failed writing to {out}: {e}");
                return;
            }

            println!("Saved to {out}");
        }

        Command::Logout => {
            let p = token_path();
            match std::fs::remove_file(&p) {
                Ok(_) => println!("Logged out. Removed {:?}", p),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    println!("Already logged out (no token file).")
                }
                Err(e) => eprintln!("Failed to remove {:?}: {e}", p),
            }
        }
    }
}

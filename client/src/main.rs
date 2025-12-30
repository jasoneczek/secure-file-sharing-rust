mod cli;
mod token_store;
mod types;

use clap::Parser;

use cli::{Cli, Command};
use token_store::*;
use types::*;

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

            let refresh_token = match require_refresh(&store) {
                Some(t) => t,
                None => {
                    eprintln!("No refresh token saved. Login again.");
                    return;
                }
            };

            let url = format!("{}/token/refresh", cli.base);

            let resp = reqwest::Client::new()
                .get(url)
                .bearer_auth(refresh_token)
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

            if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                eprintln!("{body}");
                return;
            }

            let out: UploadResp = match resp.json().await {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Failed to parse JSON: {e}");
                    return;
                }
            };

            println!(
                "Uploaded file_id={} filename={} size={} public={}",
                out.file_id, out.filename, out.size, out.is_public
            );
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

        Command::Logout => match logout_local() {
            Ok(()) => println!("Logged out. Removed {:?}", token_path()),
            Err(e) => eprintln!("Failed to remove {:?}: {e}", token_path()),
        },
    }
}

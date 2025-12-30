use clap::{Parser, Subcommand};

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
    // Test connection
    Health,
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
    }
}


use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sfs", about = "Secure File Sharing CLI client")]
pub struct Cli {
    #[arg(long, default_value = "http://localhost:8080")]
    pub base: String,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
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
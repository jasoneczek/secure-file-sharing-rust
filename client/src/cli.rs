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
#[command(about = "Commands", arg_required_else_help = true)]
pub enum Command {
    /// Check server connectivity
    Health,

    /// Register a new user account
    Register { username: String, password: String },

    /// Log in and save tokens locally
    Login { username: String, password: String },

    /// Show the current authenticated user
    Me,

    /// Refresh the access token using the refresh token
    Refresh,

    /// Upload a file
    Upload {
        /// Path to a local file
        path: String,

        /// Make the uploaded file public
        #[arg(long, default_value_t = false)]
        public: bool,
    },

    /// Download a file you have access to
    Download {
        /// File id on the server
        file_id: u32,

        /// Output path to save the file
        #[arg(long)]
        out: String,
    },

    /// Share a file with another user id
    Share {
        /// File id on the server
        file_id: u32,

        /// Target user id
        user_id: u32,
    },

    /// Revoke a share for a user id
    RevokeUser {
        /// File id on the server
        file_id: u32,

        /// Target user id
        user_id: u32,
    },

    /// Download a public file (no login required)
    PublicDownload {
        /// File id on the server
        file_id: u32,

        /// Output path to save the file
        #[arg(long)]
        out: String,
    },

    /// List files visible to the logged-in user
    #[command(alias = "files")]
    List,

    /// Remove saved tokens (log out)
    Logout,
}
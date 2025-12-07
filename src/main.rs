mod models;

fn main() {
    let max_file_size: u64 = 100_000_000;
    let port: u16 = 8080;
    let max_users: u32 = 10_000;

    println!("\n=== File Sharing Server ===");
    println!("Max file size: {} bytes ({} MB)", max_file_size, max_file_size / 1_000_000);
    println!("PORT: {}", port);
    println!("Max users: {}", max_users);

    // Test creating a user
    let test_user = models::user::User {
        id: 1,
        username: String::from("Test"),
        password_hash: String::from("hashed_password"),
        created_at: 1699564800,
    };

    println!("\nTest user created: {} at {}", test_user.username, test_user.created_at);
}

mod models;

fn main() {
    println!("\n=== File Sharing Server ===");

    let max_file_size: u64 = 100_000_000;
    let port: u16 = 8080;
    let max_users: u32 = 10_000;

    println!("Configuration:");
    println!("PORT: {}", port);
    println!("Max file size: {} MB", max_file_size / 1_000_000);
    println!("Max users: {}", max_users);

    // Test creating a user with constructor
    let mut test_user = models::user::User::new (
        1,
        String::from("alice"),
        String::from("hashed_password"),
    );

    // Display user info
    test_user.display_info();

    // Check if active
    println!("Is user active? {}", test_user.is_active());

    // Deactivate user
    test_user.deactivate();
    println!("After deactivation:");
    test_user.display_info();

    // Update password
    test_user.update_password(String::from("new_hashed_password_456"));
    println!("Password updated successfully");

    // Test creating a file
    let test_file = models::file::File::new(
        1,
        String::from("Test.pdf"),
        2_500_000,
        test_user.id,
    );

    test_file.display_info();

    println!("File size: {:.2} MB ({} KB)", test_file.size_in_mb(), test_file.size_in_kb());

    if test_file.is_owned_by(test_user.id) {
        println!("File ownership verified for user ID {}", test_user.id);
    } else {
        println!("File not owned by this user");
    }

    println!("\n=== Validation Tests ===");

    // User validation
    println!("Username valid? {}", test_user.has_valid_username());
    println!("Has password? {}", test_user.has_password());
    println!("Username length: {}", test_user.username_length());

    // File validation
    println!("\nFile has .pdf extension? {}", test_file.has_extension(".pdf"));
    println!("File exceeds {}MB limit? {}",
        max_file_size / 1_000_000,
        test_file.exceeds_size_limit(max_file_size)
    );
    println!("Filename without extension: {}", test_file.filename_without_extension());
}

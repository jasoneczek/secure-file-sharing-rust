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

    println!("\n=== User with Optional Email ===");
    let mut user = models::user::User::new (
        1,
        String::from("alice"),
        String::from("hashed_password"),
    );

    user.display_info();

    // Validate user
    match user.validate_username() {
        Ok(_) => println!("Username validation passed"),
        Err(e) => println!("Username validation failed: {:?}", e),
    }

    // Add email
    user.set_email(String::from("alice@example.com"));
    println!("\nAfter adding email:");
    user.display_info();

    println!("\n=== File with validation ===");
    let mut file = models::file::File::new(
        1,
        String::from("report.pdf"),
        2_500_000,
        user.id
    );

    file.display_info();

    // Validate file size
    match file.validate_size(max_file_size) {
        Ok(_) => println!("File size validation passed"),
        Err(models::file::FileError::ExceedsSizeLimit(size)) => {
            println!("File too large: {} bytes", size);
        },
        _ => println!("File validation failed"),
    }

    // Validate file extension
    match file.validate_extension(&[".pdf", ".txt", ".jpg"]) {
        Ok(_) => println!("File extension validation passed"),
        Err(models::file::FileError::InvalidExtension(name)) => {
            println!("Invalid file type: {}", name);
        },
        _ => println!("Extension validation failed"),
    }

    // Add description
    file.set_description(String::from("Lab Report"));
    println!("\nFile description: {}", file.get_description().unwrap_or(&String::from("None")));

    println!("\n=== Permission System ===");
    let permission = models::permission::Permission::new(
        1,
        file.id,
        user.id,
        models::permission::PermissionType::Owner
    );

    permission.display_info();

    if permission.is_owner() {
        println!("User has owner permissions");
    }
}
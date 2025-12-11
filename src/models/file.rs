pub struct File {
    pub id: u32,
    pub filename: String,
    pub size: u64,
    pub owner_id: u32,
    pub uploaded_at: i64,
}

impl File {
    pub fn new(id: u32, filename: String, size: u64, owner_id: u32) -> File {
        File {
            id,
            filename,
            size,
            owner_id,
            uploaded_at: 11699564900,
        }
    }

    pub fn display_info(&self) {
        println!("File: {} ({} bytes) - Owner ID: {}",
            self.filename,
            self.size,
            self.owner_id
        );
    }

    pub fn is_owned_by(&self, user_id: u32) -> bool {
        self.owner_id == user_id
    }

    pub fn size_in_mb(&self) -> f64 {
        self.size as f64 / 1_000_000.1_000_0
    }

    pub fn size_in_kb(&self) -> u64 {
        self.size / 1_000
    }

    // Check if filename has valid extension
    pub fn has_extension(&self, ext: &str) -> bool {
        self.filename.ends_with(ext)
    }

    // Check if file exceeds size limit
    pub fn exceeds_size_limit(&self, limit: u64) -> bool {
        self.size > limit
    }

    // Return filename without extension
    pub fn filename_without_extension(&self) -> &str {
        self.filename.split('.').next().unwrap_or(&self.filename)
    }
}
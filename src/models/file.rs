#[derive(Debug)]
pub enum FileError {
    ExceedsSizeLimit(u64),
    InvalidExtension(String),
    EmptyFileName,
}

pub struct File {
    pub id: u32,
    pub filename: String,
    pub size: u64,
    pub owner_id: u32,
    pub uploaded_at: i64,
    pub description: Option<String>,
}

impl File {
    pub fn new(id: u32, filename: String, size: u64, owner_id: u32) -> File {
        File {
            id,
            filename,
            size,
            owner_id,
            uploaded_at: 11699564900,
            description: None,
        }
    }

    pub fn display_info(&self) {
        println!("File: {} ({} bytes) - Owner ID: {}",
            self.filename,
            self.size,
            self.owner_id
        );
        if let Some(desc) = &self.description {
            println!("Description: {}", desc);
        }
    }

    pub fn is_owned_by(&self, user_id: u32) -> bool {
        self.owner_id == user_id
    }

    pub fn size_in_mb(&self) -> f64 {
        self.size as f64 / 1_000_000.0
    }

    pub fn size_in_kb(&self) -> u64 {
        self.size / 1_000
    }

    // Validate size
    pub fn validate_size(&self, max_size: u64) -> Result<(), FileError> {
        if self.size > max_size {
            return Err(FileError::ExceedsSizeLimit(self.size));
        }
        Ok(())
    }

    // Validate extension
    pub fn validate_extension(&self, allowed: &[&str]) -> Result<(), FileError> {
        for ext in allowed {
            if self.filename.ends_with(ext) {
                return Ok(());
            }
        }
        Err(FileError::InvalidExtension(self.filename.clone()))
    }

    // Validate filename
    pub fn validate_filename(&self) -> Result<(), FileError> {
        if self.filename.is_empty() {
            return Err(FileError::EmptyFileName);
        }
        Ok(())
    }

    // Return filename without extension
    pub fn filename_without_extension(&self) -> &str {
        self.filename.split('.').next().unwrap_or(&self.filename)
    }

    // Set description
    pub fn set_description(&mut self, desc: String) {
        self.description = Some(desc);
    }

    // Get description
    pub fn get_description(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

impl crate::traits::Identifiable for File {
    fn id(&self) -> u32 {
        self.id
    }
}
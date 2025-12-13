use crate::models::file::File;

pub struct FileRepository {
    files: Vec<File>,
}

impl FileRepository {
    pub fn new() -> Self {
        FileRepository { files: Vec::new() }
    }

    pub fn add(&mut self, file: File) {
        self.files.push(file);
    }

    // Find file by ID
    pub fn find_by_id(&self, id: u32) -> Option<&File> {
        self.files.iter().find(|f| f.id == id)
    }

    // Find all files owned by a specific user using filter and collect
    pub fn find_by_owner(&self, owner_id: u32) -> Vec<&File> {
        self.files
            .iter()
            .filter(|f| f.owner_id == owner_id)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }
}
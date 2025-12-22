use crate::models::file::File;
use crate::repository::{FileRepository, PermissionRepository};

/// Business logic for file operations
pub struct FileService<'a> {
    pub files: &'a FileRepository,
    pub permissions: &'a PermissionRepository,
}

impl<'a> FileService<'a> {
    pub fn new(files: &'a FileRepository, permissions: &'a PermissionRepository) -> Self {
        Self { files, permissions }
    }

    /// Check whether a user can download a file
    pub fn can_download(&self, user_id: u32, file_id: u32) -> bool {
        // Owner always allowed
        if let Some(file) = self.files.find_by_id(file_id) {
            if file.owner_id == user_id {
                return true;
            }
        }

        // Check permissions repository
        self.permissions.user_has_access(user_id, file_id)
    }

    pub fn get_for_download(&self, user_id: u32, file_id: u32) -> Option<&File> {
        let file = self.files.find_by_id(file_id)?;

        if file.is_public || file.owner_id == user_id {
            return Some(file);
        }

        if self.permissions.user_has_access(user_id, file_id) {
            return Some(file);
        }

        None
    }

    pub fn get_public_for_download(&self, file_id: u32) -> Option<&File> {
        let file = self.files.find_by_id(file_id)?;
        if file.is_public { Some(file) } else { None }
    }
}

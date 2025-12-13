use crate::models::permission::Permission;

pub struct PermissionRepository {
    permissions: Vec<Permission>,
}

impl PermissionRepository {
    pub fn new() -> Self {
        PermissionRepository { permissions: Vec::new() }
    }

    // Add permission
    pub fn add(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    // Find permission by ID
    pub fn find_by_id(&self, id: u32) -> Option<&Permission> {
        self.permissions.iter().find(|p| p.id == id)
    }

    // Get all permissions for a specific file
    pub fn find_by_file(&self, file_id: u32) -> Vec<&Permission> {
        self.permissions
            .iter()
            .filter(|p| p.file_id == file_id)
            .collect()
    }

    // Get all permissions for a specific user
    pub fn find_by_user(&self, user_id: u32) -> Vec<&Permission> {
        self.permissions
            .iter()
            .filter(|p| p.user_id == user_id)
            .collect()
    }

    // Check if user has permission to a file
    pub fn user_has_access(&self, user_id: u32, file_id: u32) -> bool {
        self.permissions
            .iter()
            .any(|p| p.user_id == user_id && p.file_id == file_id)
    }

    // Remove a permission
    pub fn remove(&mut self, id: u32) {
        self.permissions.retain(|p| p.id != id);
    }

    // Count permissions
    pub fn count(&self) -> usize {
        self.permissions.len()
    }
}
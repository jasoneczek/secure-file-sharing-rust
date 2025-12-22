pub enum PermissionType {
    Owner,
    Shared,
    Public,
}

pub struct Permission {
    pub id: u32,
    pub file_id: u32,
    pub user_id: u32,
    pub permission_type: PermissionType,
}

impl Permission {
    pub fn new(id: u32, file_id: u32, user_id: u32, permission_type: PermissionType) -> Permission {
        Permission {
            id,
            file_id,
            user_id,
            permission_type,
        }
    }

    pub fn is_owner(&self) -> bool {
        matches!(self.permission_type, PermissionType::Owner)
    }

    pub fn display_info(&self) {
        let perm_str = match self.permission_type {
            PermissionType::Owner => "Owner",
            PermissionType::Shared => "Shared",
            PermissionType::Public => "Public",
        };
        println!(
            "Permission: User {} has {} access to File {}",
            self.user_id, perm_str, self.file_id
        );
    }
}

impl crate::traits::Identifiable for Permission {
    fn id(&self) -> u32 {
        self.id
    }
}

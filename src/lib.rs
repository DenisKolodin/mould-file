#[macro_use]
extern crate mould;

pub mod file;

use mould::session::Session;

pub enum FileAccessPermission {
    CanRead,
    CanWrite,
    CanDelete,
}

pub trait HasFileAccessPermission: Session {
    fn has_permission(&self, path: &str, permission: FileAccessPermission) -> bool;
}

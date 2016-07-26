#[macro_use]
extern crate mould;

pub mod file;

use mould::session::SessionData;

pub enum FileAccessPermission {
    CanRead,
    CanWrite,
    CanDelete,
}

pub trait HasFileAccessPermission: SessionData {
    fn has_permission(&self, path: &str, permission: FileAccessPermission) -> bool;
}

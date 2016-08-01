use std::fs::File;
use std::path::Path;
use std::convert::AsRef;
use std::io::prelude::*;
use mould::prelude::*;
use permission::HasPermission;

pub enum FileAccess {
    CanRead,
    CanWrite,
    CanDelete,
}

pub struct FileAccessPermission<'a> {
    pub path: &'a Path,
    pub access: FileAccess,
}

pub struct FileService { }

impl FileService {
    pub fn new() -> Self {
        FileService { }
    }
}

impl<T> Service<T> for FileService
    where T: for <'a> HasPermission<FileAccessPermission<'a>> {
    fn route(&self, request: &Request) -> Box<Worker<T>> {
        if request.action == "read-file" {
            Box::new(FileReadWorker::new())
        } else {
            let msg = format!("Unknown action '{}' for file service!", request.action);
            Box::new(RejectWorker::new(msg))
        }
    }
}

struct FileReadWorker {
    file: Option<File>,
}

impl FileReadWorker {
    fn new() -> Self {
        FileReadWorker { file: None }
    }
}

impl<T> Worker<T> for FileReadWorker
    where T: for<'a> HasPermission<FileAccessPermission<'a>> {
    fn prepare(&mut self, context: &mut T, mut request: Request) -> worker::Result<Shortcut> {
        let path: String = try!(request.extract("path")
            .ok_or(worker::Error::reject("Path to file is not set.")));
        {
        let permission = FileAccessPermission {
            path: path.as_ref(),
            access: FileAccess::CanRead,
        };
        if context.has_permission(&permission) {
            let file = try!(File::open(&path));
            self.file = Some(file);
            Ok(Shortcut::Tuned)
        } else {
            Err(worker::Error::Reject("You haven't permissions!".to_string()))
        }
        }
    }
    fn realize(&mut self, _: &mut T, _: Option<Request>) -> worker::Result<Realize> {
        let mut file = try!(self.file.take().ok_or(
            worker::Error::reject("File handle was lost.")));
        let mut content = String::new();
        try!(file.read_to_string(&mut content));
        Ok(Realize::OneItemAndDone(mould_object!{"content" => content}))
    }
}

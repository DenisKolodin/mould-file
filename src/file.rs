use std::fs::File;
use std::io::prelude::*;
use mould::prelude::*;
use super::{FileAccessPermission, HasFileAccessPermission};

pub struct FileRouter { }

impl FileRouter {
    pub fn new() -> Self {
        FileRouter { }
    }
}

impl<CTX> Router<CTX> for FileRouter where CTX: HasFileAccessPermission {
    fn route(&self, request: &Request) -> Box<Worker<CTX>> {
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

impl<CTX> Worker<CTX> for FileReadWorker where CTX: HasFileAccessPermission {
    fn prepare(&mut self, context: &mut CTX, mut request: Request) -> worker::Result<Shortcut> {
        let path: String = try!(request.extract("path")
            .ok_or(worker::Error::reject("Path to file is not set.")));
        if context.has_permission(&path, FileAccessPermission::CanRead) {
            let file = try!(File::open(&path));
            self.file = Some(file);
            Ok(Shortcut::Tuned)
        } else {
            Err(worker::Error::Reject("You haven't permissions!".to_string()))
        }
    }
    fn realize(&mut self, _: &mut CTX, _: Option<Request>) -> worker::Result<Realize> {
        let mut file = try!(self.file.take().ok_or(
            worker::Error::reject("File handle was lost.")));
        let mut content = String::new();
        try!(file.read_to_string(&mut content));
        Ok(Realize::OneItemAndDone(mould_object!{"content" => content}))
    }
}

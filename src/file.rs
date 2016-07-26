use std::fs::File;
use std::io::prelude::*;
use mould::prelude::*;
use super::{FileAccessPermission, HasFileAccessPermission};

pub struct FileHandler { }

impl FileHandler {
    pub fn new() -> Self {
        FileHandler { }
    }
}

impl<CTX> Handler<CTX> for FileHandler where CTX: HasFileAccessPermission {
    fn build(&self, mut request: Request) -> Box<Worker<CTX>> {
        if request.action == "read-file" {
            Box::new(FileReadWorker {
                name: request.extract("name"),
                file: None,
            })
        } else {
            let msg = format!("Unknown action '{}' for file service!", request.action);
            Box::new(RejectWorker::new(msg))
        }
    }
}

struct FileReadWorker {
    name: Option<String>,
    file: Option<File>,
}

impl<CTX> Worker<CTX> for FileReadWorker where CTX: HasFileAccessPermission {
    fn shortcut(&mut self, session: &mut CTX) -> WorkerResult<Shortcut> {
        let name = try!(self.name.take().ok_or(
            WorkerError::Reject("Name of file not set.".to_string())));
        if session.has_permission(&name, FileAccessPermission::CanRead) {
            let file = try!(File::open(&name));
            self.file = Some(file);
            Ok(Shortcut::Tuned)
        } else {
            Err(WorkerError::Reject("You haven't permissions!".to_string()))
        }
    }
    fn realize(&mut self, _: &mut CTX, _: Option<Request>) -> WorkerResult<Realize> {
        let mut file = try!(self.file.take().ok_or(
            WorkerError::reject("File handle was lost.")));
        let mut content = String::new();
        try!(file.read_to_string(&mut content));
        Ok(Realize::OneItemAndDone(mould_object!{"content" => content}))
    }
}

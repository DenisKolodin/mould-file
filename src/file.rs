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
        if request.action == "read" {
            Box::new(FileReadWorker {
                name: request.extract("name"),
            })
        } else {
            let msg = format!("Unknown action '{}' for file service!", request.action);
            Box::new(RejectWorker::new(msg))
        }
    }
}

struct FileReadWorker {
    name: Option<String>,
}

impl<CTX> Worker<CTX> for FileReadWorker where CTX: HasFileAccessPermission {
    fn shortcut(&mut self, session: &mut CTX) -> WorkerResult<Shortcut> {
        let name = try!(self.name.take().ok_or(
                WorkerError::Reject("Name of file not set.".to_string())));
        if session.has_permission(&name, FileAccessPermission::CanRead) {
            Ok(Shortcut::Tuned)
        } else {
            Err(WorkerError::Reject("You haven't permissions!".to_string()))
        }
    }
}

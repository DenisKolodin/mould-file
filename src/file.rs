use std::convert::Into;
use std::fs::File;
use std::path::Path;
use std::convert::AsRef;
use std::io::prelude::*;
use permission::HasPermission;
use mould::prelude::*;
use mould::rustc_serialize::json::Json;

pub enum FileAccess {
    CanRead,
    CanWrite,
    CanDelete,
}

pub struct FileAccessPermission<'a> {
    pub path: &'a Path,
    pub access: FileAccess,
}

macro_rules! check_permission {
    ($session:ident, $path:ident, $perm:ident) => {{
        let permission = FileAccessPermission {
            path: $path.as_ref(),
            access: $crate::FileAccess::$perm,
        };
        if !$session.has_permission(&permission) {
            return Err("You haven't permissions!".into());
        }
    }};
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
            Box::new(ReadFileWorker::new(false))
        } else if request.action == "write-file" {
            Box::new(WriteFileWorker::new(false))
        } else {
            let msg = format!("Unknown action '{}' for file service!", request.action);
            Box::new(RejectWorker::new(msg))
        }
    }
}

pub struct JsonFileService { }

impl JsonFileService {
    pub fn new() -> Self {
        JsonFileService { }
    }
}

impl<T> Service<T> for JsonFileService
    where T: for <'a> HasPermission<FileAccessPermission<'a>> {
    fn route(&self, request: &Request) -> Box<Worker<T>> {
        if request.action == "read-object" {
            Box::new(ReadFileWorker::new(true))
        } else if request.action == "write-object" {
            Box::new(WriteFileWorker::new(true))
        } else {
            let msg = format!("Unknown action '{}' for json file service!", request.action);
            Box::new(RejectWorker::new(msg))
        }
    }
}


struct ReadFileWorker {
    /// Flag to convert json
    convert: bool,
    file: Option<File>,
}

impl ReadFileWorker {
    fn new(convert: bool) -> Self {
        ReadFileWorker {
            convert: convert,
            file: None,
        }
    }
}

impl<T> Worker<T> for ReadFileWorker
    where T: for<'a> HasPermission<FileAccessPermission<'a>> {
    fn prepare(&mut self, session: &mut T, mut request: Request) -> worker::Result<Shortcut> {
        let path: String = extract_field!(request, "path");
        check_permission!(session, path, CanRead);
        let file = try!(File::open(&path));
        self.file = Some(file);
        Ok(Shortcut::Tuned)
    }

    fn realize(&mut self, _: &mut T, _: Option<Request>) -> worker::Result<Realize> {
        let mut file = self.file.take().expect("File handle expected");
        let mut content = String::new();
        try!(file.read_to_string(&mut content));
        if self.convert {
            let object = try!(Json::from_str(&content));
            Ok(Realize::OneItemAndDone(mould_object!{"object" => object}))
        } else {
            Ok(Realize::OneItemAndDone(mould_object!{"content" => content}))
        }
    }
}

struct WriteFileWorker {
    /// Flag to convert json
    convert: bool,
    file: Option<File>,
    content: Option<String>,
}

impl WriteFileWorker {
    fn new(convert: bool) -> Self {
        WriteFileWorker {
            convert: convert,
            file: None,
            content: None,
        }
    }
}

impl<T> Worker<T> for WriteFileWorker
    where T: for<'a> HasPermission<FileAccessPermission<'a>> {
    fn prepare(&mut self, session: &mut T, mut request: Request) -> worker::Result<Shortcut> {
        let path: String = extract_field!(request, "path");
        let content: String = if self.convert {
            let object = extract_field!(request, "object");
            Json::Object(object).to_string()
        } else {
            extract_field!(request, "content")
        };
        check_permission!(session, path, CanWrite);
        let file = try!(File::create(&path));
        self.file = Some(file);
        self.content = Some(content);
        Ok(Shortcut::Tuned)
    }

    fn realize(&mut self, _: &mut T, _: Option<Request>) -> worker::Result<Realize> {
        let mut file = self.file.take().expect("File handle expected");
        let content = self.content.take().expect("File content expected");
        try!(file.write_all(content.as_bytes()));
        Ok(Realize::Done)
    }
}

// TODO Add WriteChunksWorker

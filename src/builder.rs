use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use super::temp_crate::TempCrate;
use super::db::Db;
use super::tasks::*;
use super::util::run_command;

#[derive(Clone)]
enum Status {
    Pending,
    Running,
    /// Succeeded with path to a doc tarball
    Succeeded(String),
    // Failed with a string describing what went wrong
    Failed(String),
}

/// Handles compiling a crate's documentation.
pub struct Builder {
    temp_crate: TempCrate,
    db: Arc<Mutex<Db>>,
    status: RwLock<Status>,
    /// Destination path where the tarball will end up
    dest_path: String,
}

impl Builder {
    // dest_dir: Destination directory in which the tarball should be placed
    //           after downloading
    pub fn new(name: &str, version: &str, db: Arc<Mutex<Db>>, dest_path: String) -> Builder {
        Builder {
            temp_crate: TempCrate::new(name, version),
            db: db,
            status: RwLock::new(Status::Pending),
            dest_path: dest_path.clone(),
        }
    }

    // Spawn a new thread to download, compile, and store the crate's docs.
    // The builder must be wrapped in an `RwLock`. The thread will acquire
    // a write lock on it, but other threads can still read it to inspect
    // its status.
    pub fn spawn(lock: RwLock<Builder>) {
        thread::spawn(move || {
            let mut builder = lock.write().unwrap();
            builder.run();
        });
    }

    fn update_status(&self, new_status: Status) {
        let mut status = self.status.write().unwrap();
        *status = new_status;
    }

    fn run(&mut self) -> Status {
        let temp_crate = &self.temp_crate;
        self.update_status(Status::Running);

        let download = DownloadTask::new(temp_crate);
        let expand   = ExpandTask::new(temp_crate);
        let doc      = DocTask::new(temp_crate);

        let result = download.run()
            .and_then(|_| expand.run())
            .and_then(|_| doc.run())
            .and_then(|doc_path| {
                self.temp_crate.cleanup().unwrap();

                let dest_path = self.dest_path.clone();
                run_command(move || {
                    Command::new("mv")
                            .arg(doc_path)
                            .arg(dest_path)
                            .output()
                })
            });

        if let Err(err) = result {
            self.update_status(Status::Failed(format!("{:?}", err)));
            let _ = write!(io::stderr(), "Error building documentation: {:?}", err);
        } else {
            self.update_status(Status::Succeeded(self.dest_path.clone()));
        }

        self.status.read().unwrap().clone()
    }
}

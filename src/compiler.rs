use std::io::{self, Write};
use std::process::Command;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
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
struct Builder {
    temp_crate: TempCrate,
    db: Arc<Mutex<Db>>,
    status: Status,
    /// Destination path where the tarball will end up
    dest_path: String,
}

impl Builder {
    // dest_dir: Destination directory in which the tarball should be placed
    //           after downloading
    fn new(name: &str, version: &str, db: Arc<Mutex<Db>>, dest_path: &str) -> Builder {
        Builder {
            temp_crate: TempCrate::new(name, version),
            db: db,
            status: Status::Pending,
            dest_path: dest_path.to_owned(),
        }
    }

    // Spawn a new thread to download, compile, and store the crate's docs.
    // The builder must be wrapped in an `RwLock`. The thread will acquire
    // a write lock on it, but other threads can still read it to inspect
    // its status.
    fn spawn(lock: RwLock<Builder>) {
        thread::spawn(move || {
            let mut builder = lock.write().unwrap();
            builder.run();
        });
    }

    fn run(&mut self) -> Status {
        self.status = Status::Running;

        let download = DownloadTask::new(&self.temp_crate);
        let expand   = ExpandTask::new(&self.temp_crate);
        let doc      = DocTask::new(&self.temp_crate);

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
            self.status = Status::Failed(format!("{:?}", err));
            let _ = write!(io::stderr(), "Error building documentation: {:?}", err);
        } else {
            self.status = Status::Succeeded(self.dest_path.clone());
        }

        self.status.clone()
    }
}
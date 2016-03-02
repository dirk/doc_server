use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use super::db::{Db, FailedModel};
use super::store::StoredCrate;
use super::tasks::*;
use super::temp_crate::TempCrate;
use super::util::run_command;

#[derive(Clone)]
pub enum Status {
    Pending,
    Running,
    /// Succeeded with path to a doc tarball
    Succeeded(String),
    // Failed with a string describing what went wrong
    Failed(TaskError),
}

/// Handles compiling a crate's documentation.
pub struct Builder {
    pub temp_crate: TempCrate,
    pub status: RwLock<Status>,
    /// Destination path where the tarball will end up
    pub dest: StoredCrate,
}

impl Builder {
    // dest_dir: Destination directory in which the tarball should be placed
    //           after downloading
    pub fn new(name: &str, version: &str, dest: StoredCrate) -> Builder {
        Builder {
            temp_crate: TempCrate::new(name, version),
            status: RwLock::new(Status::Pending),
            dest: dest.clone(),
        }
    }

    // Spawn a new thread to download, compile, and store the crate's docs.
    // The builder must be wrapped in an `RwLock`. The thread will acquire
    // a write lock on it, but other threads can still read it to inspect
    // its status.
    pub fn spawn(db: Arc<Mutex<Db>>, builder: Arc<RwLock<Builder>>) {
        {
            let mut writeable_db = db.lock().unwrap();
            writeable_db.add_build_in_progress(builder.clone());
        }

        let builder = builder.clone();

        thread::spawn(move || {
            let status = {
                let mut writeable_builder = builder.write().unwrap();
                writeable_builder.run()
            };

            let mut writeable_db = db.lock().unwrap();

            if let Status::Failed(err) = status {
                let code: i32;
                let message: String;

                match err {
                    TaskError::Command(status, stdout, stderr) => {
                        code = status.code().unwrap_or(-1);
                        message = format!("{}\n{}", stdout, stderr).trim().to_owned();
                    },
                    _ => {
                        code = -1;
                        message = "Unknown reason".to_owned();
                    }
                }

                // Model representing the error code and message
                let failed = FailedModel {
                    code: code,
                    message: message,
                };
                // Name-version pair
                let pair = {
                    let readable_builder = builder.read().unwrap();
                    readable_builder.temp_crate.pair()
                };

                let _ = writeable_db.set_failed(&pair, failed);
            }

            // Remove the builder from the list in-progrss builds
            writeable_db.remove_build_in_progress(builder.clone());
        });
    }

    fn update_status(&self, new_status: Status) -> Status {
        let mut status = self.status.write().unwrap();
        *status = new_status.clone();
        new_status
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
                let dest_path = self.dest.path();

                let mkdirp = move || {
                    Command::new("mkdir")
                            .arg("-p").arg(dest_path)
                            .output()
                };
                let mv = move || {
                    Command::new("cp")
                            .arg("-r")
                            .arg(format!("{}/", doc_path))
                            .arg(dest_path)
                            .output()
                };

                run_command(mkdirp)
                    .and_then(|_| run_command(mv))
            });

        self.temp_crate.cleanup().unwrap(); // Always cleanup!

        if let Err(err) = result {
            let _ = write!(io::stderr(), "Error building documentation: {:?}\n", err);
            self.update_status(Status::Failed(err))
        } else {
            let dest_path = self.dest.0.clone();
            self.update_status(Status::Succeeded(dest_path))
        }
    }
}

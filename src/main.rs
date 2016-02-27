extern crate hyper;
extern crate uuid;

use uuid::Uuid;
use std::io;
use std::process::{Command, ExitStatus, Output};

mod tasks;
mod util;

use self::tasks::*;

#[derive(Debug)]
pub enum TaskError {
    DownloadRequest,
    DownloadResponse,
    CommandExecute(io::Error),
    Command(ExitStatus, String),
}

pub struct TempCrate {
    name: String,
    /// Path to the expanded crate directory
    path: String,
    /// Path to the downloaded crate package file
    crate_path: String,
    /// Path to the built doc tarball
    doc_path: Option<String>,
}

impl TempCrate {
    fn with_crate_name(name: &str) -> TempCrate {
        let uuid = Uuid::new_v4();
        let path = format!("tmp/{}-{}", name, uuid.to_hyphenated_string());

        TempCrate {
            name: name.to_owned(),
            path: path.clone(),
            crate_path: format!("{}.crate", &path),
            doc_path: None,
        }
    }

    fn cleanup(&self) -> io::Result<Output> {
        Command::new("rm")
                .arg("-rf")
                .arg(self.path.clone())
                .arg(self.crate_path.clone())
                .output()
    }
}

fn main() {
    let mut temp = TempCrate::with_crate_name("metrics_distributor-0.2.1");

    let result = {
        let download = DownloadTask::new(&temp);
        let expand   = ExpandTask::new(&temp);
        let doc      = DocTask::new(&temp);

        download.run()
            .and_then(|_| expand.run())
            .and_then(|_| doc.run())
    };

    temp.cleanup().unwrap();

    match result {
        Ok(doc_path) => {
            temp.doc_path = Some(doc_path);
        },
        Err(err) => {
            panic!("Error building documentation: {:?}", err);
        },
    }
}

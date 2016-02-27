extern crate hyper;
extern crate uuid;

use hyper::client::Client;
use uuid::Uuid;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::ops::FnOnce;
use std::process::{Command, ExitStatus, Output};

#[derive(Debug)]
enum TaskError {
    DownloadRequest,
    DownloadResponse,
    CommandExecute(io::Error),
    Command(ExitStatus, String),
}

struct TempCrate {
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

struct DownloadTask<'a> {
    temp: &'a TempCrate,
}

impl<'a> DownloadTask<'a> {
    fn new(temp: &'a TempCrate) -> DownloadTask<'a> {
        DownloadTask {
            temp: temp,
        }
    }

    fn run(&self) -> Result<(), TaskError> {
        let client = Client::new();

        let dl_url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/metrics_distributor/metrics_distributor-0.2.1.crate";
        let crate_path = self.temp.crate_path.clone();

        let dl_response = try! {
            client.get(dl_url).send()
                .map_err(|_| TaskError::DownloadRequest)
        };

        let mut response_reader = BufReader::new(dl_response);
        let mut file_writer = BufWriter::new(File::create(crate_path).unwrap());

        try! {
            io::copy(&mut response_reader, &mut file_writer)
                .map_err(|_| TaskError::DownloadResponse)
        };

        Ok(())
    }
}

struct ExpandTask<'a> {
    temp: &'a TempCrate,
}

impl<'a> ExpandTask<'a> {
    fn new(temp: &'a TempCrate) -> ExpandTask<'a> {
        ExpandTask {
            temp: temp,
        }
    }

    fn run(&self) -> Result<(), TaskError> {
        let crate_path = self.temp.crate_path.clone();
        let path = self.temp.path.clone();

        let mkdirp = move || {
            Command::new("mkdir")
                    .arg("-p").arg(path)
                    .output()
        };

        let path = self.temp.path.clone(); // Clone again for safety
        let tar = move || {
            Command::new("tar")
                    .arg("xf").arg(crate_path)
                    .arg("-C").arg(path)
                    .arg("--strip-components").arg("1")
                    .output()
        };

        run_command(mkdirp)
            .and_then(|_| run_command(tar))
    }
}

fn run_command<F>(command: F) -> Result<(), TaskError>
    where F: FnOnce() -> io::Result<Output> {
    let output = command();

    output
        .map_err(|err| TaskError::CommandExecute(err))
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                let status = output.status;
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                Err(TaskError::Command(status, stderr))
            }
        })
}

struct DocTask<'a> {
    temp: &'a TempCrate,
}

impl<'a> DocTask<'a> {
    fn new(temp: &'a TempCrate) -> DocTask<'a> {
        DocTask {
            temp: temp,
        }
    }

    fn run(&self) -> Result<String, TaskError> {
        let path = self.temp.path.clone();

        let command = format!("docker run -it --rm -v \"$(pwd)/{}:/source\" doc_server:build /home/build-doc.sh", &path);

        let doc = || {
            Command::new("/bin/sh")
                    .arg("-c")
                    .arg(command)
                    .spawn()
                    .and_then(|c| c.wait_with_output())
        };

        let tarball_path = format!("{}/doc.tar", path);
        let doc_path = &format!("tmp/doc-{}.tar", self.temp.name);

        let move_tarball = move || {
            Command::new("mv")
                    .arg(tarball_path)
                    .arg(doc_path)
                    .output()
        };

        run_command(doc)
            .and_then(|_| run_command(move_tarball))
            .and_then(|_| Ok(doc_path.clone()))
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

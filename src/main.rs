extern crate hyper;
extern crate uuid;

use hyper::client::Client;
use uuid::Uuid;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::process::{Command, Output};

enum TaskError {
    DownloadRequest,
    DownloadResponse,
    CargoDoc,
}

struct TempCrate {
    /// Path to the expanded crate directory
    path: String,
    /// Path to the downloaded crate package file
    crate_path: String,
}

impl TempCrate {
    fn with_crate_name(name: &str) -> TempCrate {
        let uuid = Uuid::new_v4();
        let path = format!("tmp/{}-{}", name, uuid.to_hyphenated_string());

        TempCrate {
            path: path.clone(),
            crate_path: format!("{}.crate", &path),
        }
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

    fn run(&self) {
        let crate_path = self.temp.crate_path.clone();
        let path = self.temp.path.clone();

        let mkdirp = Command::new("mkdir")
                             .arg("-p").arg(&path)
                             .output()
                             .unwrap();

        ExpandTask::assert_passed(mkdirp);

        let tar = Command::new("tar")
                          .arg("xf").arg(crate_path)
                          .arg("-C").arg(path)
                          .arg("--strip-components").arg("1")
                          .output()
                          .unwrap();

        ExpandTask::assert_passed(tar);
    }

    fn assert_passed(output: Output) {
        if output.status.success() { return }

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Command failed");
    }
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

    fn run(&self) -> Result<(), TaskError> {
        let path = self.temp.path.clone();

        let doc = Command::new("cargo")
                          .arg("doc")
                          .current_dir(&path)
                          .spawn()
                          .and_then(|mut c| c.wait());

        match doc {
            Ok(_) => Ok(()),
            Err(_) => Err(TaskError::CargoDoc),
        }
    }
}

fn main() {
    let temp = TempCrate::with_crate_name("metrics_distributor-0.2.1");

    let download = DownloadTask::new(&temp);
    let expand   = ExpandTask::new(&temp);
    let doc      = DocTask::new(&temp);

    download.run();
    expand.run();
    doc.run();
}

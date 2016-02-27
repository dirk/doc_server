use std::process::Command;

use super::super::{TaskError, TempCrate};
use super::super::util::run_command;

pub struct DocTask<'a> {
    temp: &'a TempCrate,
}

impl<'a> DocTask<'a> {
    pub fn new(temp: &'a TempCrate) -> DocTask<'a> {
        DocTask {
            temp: temp,
        }
    }

    pub fn run(&self) -> Result<String, TaskError> {
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

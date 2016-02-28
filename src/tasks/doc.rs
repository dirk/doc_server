use std::process::Command;

use super::TaskError;
use super::super::TempCrate;
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

        let doc_path = &format!("{}/target/doc", self.temp.path);

        run_command(doc)
            .and_then(|_| Ok(doc_path.clone()))
    }
}

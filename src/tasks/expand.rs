use std::process::Command;

use super::super::{TaskError, TempCrate};
use super::super::util::run_command;

pub struct ExpandTask<'a> {
    temp: &'a TempCrate,
}

impl<'a> ExpandTask<'a> {
    pub fn new(temp: &'a TempCrate) -> ExpandTask<'a> {
        ExpandTask {
            temp: temp,
        }
    }

    pub fn run(&self) -> Result<(), TaskError> {
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

#![allow(dead_code)]

use std::io;
use std::ops::FnOnce;
use std::process::Output;

use super::tasks::TaskError;

pub fn run_command<F>(command: F) -> Result<(), TaskError>
    where F: FnOnce() -> io::Result<Output> {
    let output = command();

    output
        .map_err(|err| {
            TaskError::CommandExecute(format!("{}", err))
        })
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                let status = output.status;
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                Err(TaskError::Command(status, stdout, stderr))
            }
        })
}

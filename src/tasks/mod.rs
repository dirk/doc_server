#![allow(dead_code)]

use std::process::ExitStatus;

mod doc;
mod download;
mod expand;

pub use self::download::DownloadTask;
pub use self::expand::ExpandTask;
pub use self::doc::DocTask;

#[derive(Clone, Debug)]
pub enum TaskError {
    DownloadRequest,
    DownloadResponse,
    CommandExecute(String),
    Command(ExitStatus, String, String),
}

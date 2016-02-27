#![allow(dead_code)]

use std::io;
use std::process::ExitStatus;

mod doc;
mod download;
mod expand;

pub use self::download::DownloadTask;
pub use self::expand::ExpandTask;
pub use self::doc::DocTask;

#[derive(Debug)]
pub enum TaskError {
    DownloadRequest,
    DownloadResponse,
    CommandExecute(io::Error),
    Command(ExitStatus, String),
}

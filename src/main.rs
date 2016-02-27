extern crate hyper;
extern crate uuid;

use std::process::{Command, ExitStatus, Output};

mod tasks;
mod temp_crate;
mod util;

use self::tasks::*;

pub use self::temp_crate::TempCrate;

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

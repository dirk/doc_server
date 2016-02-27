use hyper::client::Client;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

use super::super::{TaskError, TempCrate};

pub struct DownloadTask<'a> {
    temp: &'a TempCrate,
}

impl<'a> DownloadTask<'a> {
    pub fn new(temp: &'a TempCrate) -> DownloadTask<'a> {
        DownloadTask {
            temp: temp,
        }
    }

    pub fn run(&self) -> Result<(), TaskError> {
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

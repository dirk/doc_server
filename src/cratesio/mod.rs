use hyper::client::{Client as HyperClient};
use rustc_serialize::json::Json;
use std::error;
use std::fmt;
use std::io::BufReader;

pub struct Client {
    client: HyperClient,
}

#[derive(Debug)]
pub struct Error(pub String);

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Metadata {
    pub versions: Vec<String>,
}

fn format_error<E>(err: E) -> Error
    where E: error::Error {
    Error(format!("{:?}", err))
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: HyperClient::new(),
        }
    }

    pub fn get_crate(&self, name: &str) -> Result<Metadata, Error> {
        let url = Client::url(format!("/crates/{}", name));

        self.client.get(&url)
            .send().map_err(format_error)
            .and_then(|response| {
                let mut reader = BufReader::new(response);
                Json::from_reader(&mut reader).map_err(format_error)
            })
            .and_then(|json| {
                let versions: Vec<String> = json.find("versions").unwrap()
                    .as_array().unwrap()
                    .iter().map(|json| {
                        json.find("num").unwrap()
                            .as_string().unwrap()
                            .to_owned()
                    }).collect();

                Ok(Metadata {
                    versions: versions,
                })
            })
    }

    fn url(path: String) -> String {
        format!("https://crates.io/api/v1{}", path)
    }
}

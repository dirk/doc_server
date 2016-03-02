use iron::prelude::Request;
use plugin::Extensible;
use router::Router;
use route_recognizer::Params;
use std::sync::{Arc, Mutex};

use super::super::cratesio::{Client, Error, Metadata};
use super::super::db::Db;

pub trait GetRouter {
    fn get_router(&self) -> &Params;
}

impl<'a, 'b> GetRouter for Request<'a, 'b> {
    fn get_router<'c>(&'c self) -> &'c Params {
        self.extensions().get::<Router>().unwrap()
    }
}

pub fn get_crate(db: &Arc<Mutex<Db>>, name: &str) -> Result<Metadata, Error> {
    let db = db.lock().unwrap();

    db.get_crate(name.clone(), || {
        Client::new().get_crate(name)
    }, Some(300))
}

pub fn get_name_and_version<'a>(request: &'a Request) -> (&'a str, &'a str) {
    let name = request.get_router().find("name").unwrap();
    let version = request.get_router().find("version").unwrap();

    (name, version)
}

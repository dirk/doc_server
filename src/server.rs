extern crate hyper;
extern crate iron;
extern crate persistent;
extern crate plugin;
extern crate redis;
extern crate router;
extern crate route_recognizer;
extern crate rustc_serialize;
extern crate uuid;

use hyper::method::Method;
use iron::prelude::*;
use persistent::{Read, Write};
use router::Router;
use std::env;

pub mod cratesio;
mod compiler;
mod db;
mod store;
mod tasks;
mod temp_crate;
mod util;
mod web;

pub use self::temp_crate::TempCrate;
use db::Db;
use store::Store;

fn main() {
    use self::web::api;
    use self::web::frontend;

    let db = Db::new("redis://127.0.0.1/");

    let cwd = env::current_dir().unwrap();
    let store = Store::new(format!("{}/docs", cwd.display()));

    let mut router = Router::new();
    router.route(Method::Get, "/api/v1/crates/:name", api::get_crate);
    router.route(Method::Get, "/crates/:name/:version", frontend::get_docs);

    let mut chain = Chain::new(router);
    chain.link_before(Write::<Db>::one(db));
    chain.link_before(Read::<Store>::one(store));

    Iron::new(chain).http("localhost:3000").unwrap();
}

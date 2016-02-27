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
use persistent::Write;
use router::Router;

pub mod cratesio;
mod db;
mod tasks;
mod temp_crate;
mod util;
mod web;

pub use self::temp_crate::TempCrate;
use db::Db;

fn main() {
    use self::web::api;

    let db = Db::new("redis://127.0.0.1/");

    let mut router = Router::new();
    router.route(Method::Get, "/api/v1/crates/:name", api::get_crate);

    let mut chain = Chain::new(router);
    chain.link_before(Write::<Db>::one(db));

    Iron::new(chain).http("localhost:3000").unwrap();
}

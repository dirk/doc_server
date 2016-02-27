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
use iron::status;
use persistent::Write;
use router::Router;
use rustc_serialize::json;

pub mod cratesio;
mod db;
mod tasks;
mod temp_crate;
mod util;
mod web;

pub use self::temp_crate::TempCrate;
use cratesio::Client;
use db::{Db, GetDb};
use web::GetRouter;

fn get_crate(request: &mut Request) -> IronResult<Response> {
    let ref name = request.get_router().find("name").unwrap();

    let metadata = {
        let db = request.get_db().lock().unwrap();

        db.get_crate(name.clone(), || {
            Client::new().get_crate(name)
        }, Some(300))
    };

    match metadata {
        Ok(metadata) => {
            Ok(Response::with((
                status::Ok,
                json::encode(&metadata).unwrap()
            )))
        },
        Err(_) => {
            Ok(Response::with((status::NotFound)))
        },
    }
}

fn main() {
    let db = Db::new("redis://127.0.0.1/");

    let mut router = Router::new();
    router.route(Method::Get, "/api/v1/crates/:name", get_crate);

    let mut chain = Chain::new(router);
    chain.link_before(Write::<Db>::one(db));

    Iron::new(chain).http("localhost:3000").unwrap();
}

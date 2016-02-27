extern crate hyper;
extern crate iron;
extern crate uuid;
extern crate router;
extern crate route_recognizer;
extern crate rustc_serialize;

use hyper::method::Method;
use iron::prelude::*;
use router::Router;
use route_recognizer::Params;

pub mod cratesio;
mod tasks;
mod temp_crate;
mod util;

pub use self::temp_crate::TempCrate;
use cratesio::Client;
use rustc_serialize::json;

trait GetRouter {
    fn get_router(&mut self) -> &Params;
}

impl<'a, 'b> GetRouter for Request<'a, 'b> {
    fn get_router<'c>(&'c mut self) -> &'c Params {
        self.extensions.get::<Router>().unwrap()
    }
}

fn get_crate(request: &mut Request) -> IronResult<Response> {
    let ref name = request.get_router().find("name").unwrap();

    let metadata = Client::new().get_crate(name);

    match metadata {
        Ok(metadata) => {
            Ok(Response::with((
                iron::status::Ok,
                json::encode(&metadata).unwrap()
            )))
        },
        Err(_) => {
            Ok(Response::with((iron::status::NotFound)))
        },
    }
}

fn main() {
    let mut router = Router::new();
    router.route(Method::Get, "/api/v1/crates/:name", get_crate);

    Iron::new(router).http("localhost:3000").unwrap();
}

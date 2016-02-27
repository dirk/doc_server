use iron::prelude::Request;
use plugin::Extensible;
use router::Router;
use route_recognizer::Params;

pub trait GetRouter {
    fn get_router(&self) -> &Params;
}

impl<'a, 'b> GetRouter for Request<'a, 'b> {
    fn get_router<'c>(&'c self) -> &'c Params {
        self.extensions().get::<Router>().unwrap()
    }
}

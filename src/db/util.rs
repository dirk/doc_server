use iron::prelude::Request;
use persistent::Write;
use plugin::Extensible;
use std::sync::{Arc, Mutex};

use super::Db;

pub trait GetDb {
    fn get_db(&self) -> &Arc<Mutex<Db>>;
}

impl<'a, 'b> GetDb for Request<'a, 'b> {
    fn get_db<'c>(&'c self) -> &'c Arc<Mutex<Db>> {
        self.extensions().get::<Write<Db>>().unwrap()
    }
}

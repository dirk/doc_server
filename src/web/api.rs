use iron::prelude::*;
use iron::status;
use rustc_serialize::json;

use super::super::cratesio::Client;
use super::super::db::GetDb;
use super::super::web::GetRouter;

pub fn get_crate(request: &mut Request) -> IronResult<Response> {
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

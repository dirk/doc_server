use iron::prelude::*;
use iron::status;
use rustc_serialize::json;

use super::super::cratesio::Client;
use super::super::db::GetDb;
use super::super::store::GetStore;
use super::super::web::GetRouter;
use super::util;

pub fn get_docs(request: &mut Request) -> IronResult<Response> {
    let ref name = request.get_router().find("name").unwrap().to_owned();
    let ref version = request.get_router().find("version").unwrap().to_owned();
    let store = request.get_store();

    let metadata = util::get_crate(request.get_db(), name);

    if let Err(_) = metadata {
        return Ok(Response::with((status::NotFound)))
    }
    let metadata = metadata.unwrap();

    if !metadata.versions.contains(&version.to_owned()) {
        return Ok(Response::with((status::NotFound)))
    }

    let krate = store.make_crate(name, version);

    Ok(Response::with((
        status::Ok,
        krate.0
    )))
}

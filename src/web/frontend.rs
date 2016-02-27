use iron::prelude::*;
use iron::status;
use std::sync::RwLock;

use super::super::builder::Builder;
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

    let downloaded = store.contains(&krate);
    let downloading = false;

    if !downloaded && !downloading {
        let db = request.get_db().clone();
        let builder = Builder::new(name, version, db, krate.0.clone());

        Builder::spawn(RwLock::new(builder));

        return Ok(Response::with((
            status::Ok,
            format!("Building {} version {}...", name, version)
        )))
    }

    Ok(Response::with((
        status::Ok,
        krate.0
    )))
}

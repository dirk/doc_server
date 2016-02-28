use iron::prelude::*;
use iron::status;
use std::sync::{Arc, RwLock};

use super::super::builder::Builder;
use super::super::db::GetDb;
use super::super::store::GetStore;
use super::super::web::GetRouter;
use super::util;

pub fn get_docs(request: &mut Request) -> IronResult<Response> {
    let ref name = request.get_router().find("name").unwrap().to_owned();
    let ref version = request.get_router().find("version").unwrap().to_owned();

    let db = request.get_db().clone();
    let store = request.get_store();

    let metadata = util::get_crate(request.get_db(), name);

    if let Err(_) = metadata {
        return Ok(Response::with((status::NotFound)))
    }

    if !metadata.unwrap().versions.contains(&version.to_owned()) {
        return Ok(Response::with((status::NotFound)))
    }

    let krate = store.make_crate(name, version);

    let downloaded  = store.contains(&krate);
    let downloading = { db.lock().unwrap().is_build_in_progress(&krate) };

    match (downloaded, downloading) {
        // Not downloaded or downloading, so start a new download and build
        (false, false) => {
            let builder = Builder::new(name, version, krate.clone());

            Builder::spawn(db, Arc::new(RwLock::new(builder)));

            Ok(Response::with((
                status::Ok,
                format!("Building {} version {}...", name, version)
            )))
        },
        // Already downloading/building
        (false, true) => {
            Ok(Response::with((
                status::Ok,
                format!("Already building {} version {}...", name, version)
            )))
        },
        // Downloaded
        (true, false) => {
            Ok(Response::with((
                status::Ok,
                krate.0
            )))
        },
        _ => {
            panic!("Unreachable state: downloaded = {:?}, downloading = {:?}",
                   downloaded,
                   downloading)
        }
    }
}

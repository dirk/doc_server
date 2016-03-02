use iron::prelude::*;
use iron::status;
use rustc_serialize::json::{self, Json, ToJson};
use std::collections::BTreeMap;

use super::super::cratesio::Client;
use super::super::db::GetDb;
use super::super::store::GetStore;
use super::super::web::GetRouter;
use super::util::get_name_and_version;

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

pub fn get_crate_status(request: &mut Request) -> IronResult<Response> {
    let (name, version) = get_name_and_version(request);
    let db = request.get_db().clone();
    let store = request.get_store();

    let krate = store.make_crate(name, version);

    let downloaded = store.contains(&krate);
    let (downloading, failed) = {
        let pair = format!("{}-{}", name, version);
        let db = db.lock().unwrap();
        (db.is_build_in_progress(&krate), db.get_failed(&pair).is_some())
    };

    let status = match (downloaded, downloading, failed) {
        // Failed to download
        (false, false, true) => "failed",
        // Not downloaded or downloading
        (false, false, false) => "missing",
        // Already downloading/building
        (false, true, false) => "downloading",
        // Downloaded
        (true, false, false) => "downloaded",
        _ => {
            panic!("Unreachable state: downloaded = {:?}, downloading = {:?}, failed = {:?}",
                   downloaded,
                   downloading,
                   failed)
        }
    };

    let mut body: BTreeMap<String, Json> = BTreeMap::new();
    body.insert("status".to_owned(), status.to_json());

    Ok(Response::with((
        status::Ok,
        json::encode(&body).unwrap()
    )))
}

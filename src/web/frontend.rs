use iron::prelude::*;
use iron::{headers, status};
use mime_guess;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::{Arc, RwLock};
use tar::{self, Archive};

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

pub fn get_doc_file(request: &mut Request) -> IronResult<Response> {
    let ref name = request.get_router().find("name").unwrap().to_owned();
    let ref version = request.get_router().find("version").unwrap().to_owned();
    let ref requested_path = request.get_router().find("path").unwrap().to_owned();
    let store = request.get_store();

    let krate = store.make_crate(name, version);

    if !store.contains(&krate) {
        return Ok(Response::with((status::NotFound)))
    }

    let file = File::open(krate.0.clone()).unwrap();
    let archive = Archive::new(file);
    let entries = archive.entries().unwrap();

    for file in entries {
        let file = file.unwrap(); // Ensure there wasn't an I/O error
        let path = get_relative_file_path(&file);

        let mime_type = Path::new(&path).extension()
            .map(|e| mime_guess::get_mime_type(e.to_str().unwrap()));

        if path == requested_path.clone() {
            let mut buffer: Vec<u8> = vec![];
            let mut reader = BufReader::new(file);
            reader.read_to_end(&mut buffer).unwrap();

            let mut response = Response::with((
                status::Ok,
                buffer
            ));

            if let Some(m) = mime_type {
                response.headers.set(headers::ContentType(m))
            }

            return Ok(response)
        }
    }

    Ok(Response::with((
        status::NotFound,
        format!("Path {} not found in crate documentation", requested_path)
    )))
}

fn get_relative_file_path(file: &tar::Entry<File>) -> String {
    let path_buf = file.header().path().unwrap().into_owned();

    let mut path = path_buf.to_str().unwrap();
    if path.starts_with("./") {
        path = &path[2..];
    }

    path.to_owned()
}

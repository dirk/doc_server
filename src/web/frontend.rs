use handlebars_iron::Template;
use iron::prelude::*;
use iron::modifiers::Redirect;
use iron::status;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use super::super::builder::Builder;
use super::super::db::GetDb;
use super::super::store::GetStore;
use super::super::web::GetRouter;
use super::util::{self, get_name_and_version};

pub fn get_index(request: &mut Request) -> IronResult<Response> {
    Ok(Response::with((
        status::Ok,
        Template::new("index", hashmap!{
            "title".to_owned() => "Rust Crates documentation".to_owned(),
        })
    )))
}

pub fn get_docs(request: &mut Request) -> IronResult<Response> {
    let (name, version) = get_name_and_version(request);
    let db = request.get_db().clone();
    let store = request.get_store();

    let metadata = util::get_crate(request.get_db(), &name);

    if let Err(_) = metadata {
        return Ok(Response::with((status::NotFound)))
    }

    if !metadata.unwrap().versions.contains(&version.to_owned()) {
        return Ok(Response::with((status::NotFound)))
    }

    let pair = format!("{}-{}", name.clone(), version.clone());
    let failed = { db.lock().unwrap().get_failed(&pair) };
    if let Some(failed) = failed {
        return Ok(Response::with((
            status::Ok,
            format!("Unable to build {}:\n\n{}", pair, failed.message)
        )))
    }

    let krate = store.make_crate(&name, &version);

    let downloaded  = store.contains(&krate);
    let downloading = { db.lock().unwrap().is_build_in_progress(&krate) };

    match (downloaded, downloading) {
        // Not downloaded or downloading, so start a new download and build
        (false, false) => {
            let builder = Builder::new(&name, &version, krate.clone());

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
            let mut url = request.url.clone();
            url.path.push(name.to_owned());
            url.path.push("index.html".to_owned());

            return Ok(Response::with((status::Found, Redirect(url))))
        },
        _ => {
            panic!("Unreachable state: downloaded = {:?}, downloading = {:?}",
                   downloaded,
                   downloading)
        }
    }
}

pub fn get_doc_file(request: &mut Request) -> IronResult<Response> {
    let (name, version) = get_name_and_version(request);
    let ref requested_path = sanitize_requested_path(request.get_router().find("path").unwrap());
    let store = request.get_store();

    let krate = store.make_crate(&name, &version);

    if !store.contains(&krate) {
        return Ok(Response::with((status::NotFound)))
    }

    let mut path_buf = PathBuf::from(krate.0);
    path_buf.extend(Path::new(requested_path));

    if path_buf.is_file() {
        return Ok(Response::with((
            status::Ok,
            path_buf.as_path()
        )))
    }

    // Check if we can serve an "index.html"
    let index_path_buf = path_buf.join(Path::new("index.html"));
    if index_path_buf.is_file() {
        let mut index_url = request.url.clone();
        // Remove a trailing slash if found
        if index_url.path.last().unwrap() == "" {
            index_url.path.pop();
        }
        index_url.path.push("index.html".to_owned());

        return Ok(Response::with((
            status::Found,
            Redirect(index_url)
        )))
    }

    Ok(Response::with((
        status::NotFound,
        format!("Path {} not found in crate documentation", requested_path)
    )))
}

fn sanitize_requested_path(path: &str) -> String {
    let mut path = path;

    // Strip off a trailing slash
    if path.ends_with("/") {
        path = &path[..path.len()-1];
    }

    path.to_owned()
}

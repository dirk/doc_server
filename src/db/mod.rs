use iron::typemap;
use redis::{self, Commands, RedisError};
use rustc_serialize::json;
use rustc_serialize::{Encodable, Decodable};
use std::collections::HashMap;
use std::error;
use std::sync::{Arc, RwLock};

use super::builder::Builder;
use super::cratesio::{Error, Metadata};
use super::store::StoredCrate;

pub mod util;

pub use self::util::GetDb;

pub struct Db {
    redis_con: redis::Connection,

    builds_in_progress: HashMap<StoredCrate, Arc<RwLock<Builder>>>,
}

/// Represents a failed crate documentation build
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct FailedModel {
    pub code: i32,
    pub message: String,
}

impl typemap::Key for Db { type Value = Db; }

impl Db {
    pub fn new(redis_url: &str) -> Db {
        let client = redis::Client::open(redis_url).unwrap();
        let con    = client.get_connection().unwrap();

        Db {
            redis_con: con,
            builds_in_progress: HashMap::new(),
        }
    }

    // krate: Name-version pair
    pub fn set_failed(&self, krate: &str, failed: FailedModel) -> Result<(), Error> {
        let key = format!("failed:{}", krate);
        let data = json::encode(&failed).unwrap();

        let result = self.redis_con.set(key.clone(), data)
            .map_err(|err| Error(format!("{}", err)));

        let _: Result<(), RedisError> = self.redis_con.expire(key, 60 * 60 * 24); // Expire after a day

        result
    }

    pub fn get_failed(&self, krate: &str) -> Option<FailedModel> {
        let key = format!("failed:{}", krate);
        let result: Option<String> = self.redis_con.get(key).unwrap();

        result.map(|data| {
            json::decode::<FailedModel>(&data).unwrap()
        })
    }

    // expire_in: Also set time-to-live in second
    pub fn get_crate<F>(&self, name: &str, fetch: F, expire_in: Option<usize>) -> Result<Metadata, Error>
        where F: FnOnce() -> Result<Metadata, Error> {
        let key = format!("crate:{}", name.clone());

        let mut did_fetch = false;

        let metadata = self.fetch(key.clone(), move || {
            did_fetch = true;
            fetch()
        });

        if did_fetch && expire_in.is_some() {
            let _: () = self.redis_con.expire(key, expire_in.unwrap()).unwrap();
        }

        metadata
    }

    pub fn add_build_in_progress(&mut self, builder: Arc<RwLock<Builder>>) {
        let dest = builder.read().unwrap().dest.clone();

        self.builds_in_progress.insert(dest, builder);
    }

    pub fn remove_build_in_progress(&mut self, builder: Arc<RwLock<Builder>>) {
        let ref dest = builder.read().unwrap().dest;

        self.builds_in_progress.remove(dest);
    }

    pub fn is_build_in_progress(&self, krate: &StoredCrate) -> bool {
        self.builds_in_progress.contains_key(krate)
    }

    fn fetch<F, T, E>(&self, key: String, fetch: F) -> Result<T, E>
        where F: FnOnce() -> Result<T, E>,
              T: Encodable + Decodable,
              E: error::Error {
        let existing: Option<String> = self.redis_con.get(key.clone()).unwrap();

        match existing {
            Some(data) => {
                Ok(json::decode::<T>(&data).unwrap())
            },
            None => {
                let result = fetch();

                result.map(|value| {
                    let data = json::encode(&value).unwrap();
                    let _: () = self.redis_con.set(key, data).unwrap();
                    value
                })
            }
        }
    }
}

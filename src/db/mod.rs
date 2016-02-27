use iron::typemap;
use redis::{self, Commands};
use rustc_serialize::json;
use rustc_serialize::{Encodable, Decodable};
use std::error;

use super::cratesio::{Error, Metadata};

pub mod util;

pub use self::util::GetDb;

pub struct Db {
    redis_con: redis::Connection,
}

impl typemap::Key for Db { type Value = Db; }

impl Db {
    pub fn new(redis_url: &str) -> Db {
        let client = redis::Client::open(redis_url).unwrap();
        let con    = client.get_connection().unwrap();

        Db {
            redis_con: con,
        }
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

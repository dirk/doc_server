use iron::prelude::Request;
use iron::typemap;
use persistent::Read;
use plugin::Extensible;
use std::path::Path;
use std::sync::Arc;

pub struct Store {
    // Directory where the doc tarballs are stored
    path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct StoredCrate(pub String);

impl StoredCrate {
    pub fn path<'a>(&'a self) -> &'a Path {
        Path::new(&self.0)
    }
}

impl Store {
    pub fn new(path: String) -> Store {
        Store {
            path: path,
        }
    }

    pub fn contains(&self, krate: &StoredCrate) -> bool {
        let path = Path::new(&krate.0);
        path.exists()
    }

    pub fn make_crate(&self, name: &str, version: &str) -> StoredCrate {
        StoredCrate(format!("{}/{}/{}", self.path, name, version))
    }
}

impl typemap::Key for Store { type Value = Store; }

pub trait GetStore {
    fn get_store(&self) -> &Arc<Store>;
}

impl<'a, 'b> GetStore for Request<'a, 'b> {
    fn get_store<'c>(&'c self) -> &'c Arc<Store> {
        self.extensions().get::<Read<Store>>().unwrap()
    }
}

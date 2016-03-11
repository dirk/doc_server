use iron::prelude::Request;
use iron::typemap;
use persistent::Read;
use plugin::Extensible;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub struct Store {
    // Directory where the doc tarballs are stored
    pub path: String,
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

    /// Returns a `Vec` of crate names (directories) in this store.
    pub fn crate_names(&self) -> Vec<String> {
        Store::get_dir_entries(self.path.clone())
    }

    fn get_dir_entries<P>(dir: P) -> Vec<String>
        where P: AsRef<Path> {
        let mut entries: Vec<String> = vec![];

        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();

            if fs::metadata(&path).unwrap().is_dir() {
                let file_name = path.file_name().unwrap();
                entries.push(file_name.to_str().unwrap().to_owned())
            }
        }

        entries
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

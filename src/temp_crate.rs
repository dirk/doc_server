use std::io;
use std::process::{Command, Output};
use uuid::Uuid;

pub struct TempCrate {
    pub name: String,
    pub version: String,
    /// Path to the expanded crate directory
    pub path: String,
    /// Path to the downloaded crate package file
    pub crate_path: String,
}

impl TempCrate {
    pub fn new(name: &str, version: &str) -> TempCrate {
        let uuid = Uuid::new_v4();
        let path = format!("tmp/{}-{}-{}", name, version, uuid.to_hyphenated_string());

        TempCrate {
            name: name.to_owned(),
            version: version.to_owned(),
            path: path.clone(),
            crate_path: format!("{}.crate", &path),
        }
    }

    pub fn cleanup(&self) -> io::Result<Output> {
        Command::new("rm")
                .arg("-rf")
                .arg(self.path.clone())
                .arg(self.crate_path.clone())
                .output()
    }
}

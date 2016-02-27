pub struct TempCrate {
    pub name: String,
    /// Path to the expanded crate directory
    pub path: String,
    /// Path to the downloaded crate package file
    pub crate_path: String,
    /// Path to the built doc tarball
    pub doc_path: Option<String>,
}

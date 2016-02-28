# doc_server

**Note**: This is a work-in-progress attempt at building a Rust documentation server capable of downloading crates, compiling those crates' documentation in a background thread, and serving crates' documentation to users.

### How it works

This is designed to have a user experience similar to [RubyDoc.info](http://www.rubydoc.info/): providing a copy of the generated documentation for the requested name-and-version of a public Ruby gem/Rust crate.

Generated documentation is stored on the local file-system (eg. `docs/foo-1.2.3`), if the requested documentation is not available the server starts a background thread that does the following:

1. Downloads a `.crate` from the crates.io Amazon S3 archive.
2. Expands the crate archive (it's really just a tarball).
3. Starts an isolated Docker container in that expanded archive (using the stable version of Rust compiler) and calls `cargo doc` in that container.
4. Upon success it moves the generated doc folder into the storage directory; upon failure it records the reason in a local Redis instance.

#### License

Licensed under the 3-clause BSD license. See [LICENSE](LICENSE) for details.

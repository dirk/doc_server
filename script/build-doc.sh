#!/bin/bash

# Runs inside the docker container and builds the crate's documentation.

# Build the crate's documentation.
cargo doc -q

# Create a `doc.tar` at the top-level will all the documentation files at
# the root of the tarball (ie. no prefix).
tar -c -f doc.tar -C target/doc .

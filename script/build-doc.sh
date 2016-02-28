#!/bin/bash

# Runs inside the docker container and builds the crate's documentation.

# Build the crate's documentation.
cargo doc -q --color never

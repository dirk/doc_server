#!/bin/bash

# Used to build the docker image for compiling crate docs.

docker build --tag=doc_server:build --file support/Dockerfile .

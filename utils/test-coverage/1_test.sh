#! /bin/sh

set -e

export RUSTFLAGS="-C instrument-coverage"
cargo test

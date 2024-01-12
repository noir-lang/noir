#!/bin/bash

fixtures_dir="./compiler/wasm/test/fixtures"

nargo compile --program-dir=$fixtures_dir/noir-contract
nargo compile --program-dir=$fixtures_dir/simple
nargo compile --program-dir=$fixtures_dir/with-deps

#!/bin/bash
set -eu

cd /usr/src/noir
yarn workspace @noir-lang/backend_barretenberg build
yarn workspace @noir-lang/backend_barretenberg test
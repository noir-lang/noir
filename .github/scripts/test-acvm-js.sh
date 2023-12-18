#!/bin/bash
set -eu

cd /usr/src/noir
apt-get install -y jq
yarn workspace @noir-lang/acvm_js build
yarn workspace @noir-lang/acvm_js test
#!/bin/bash
set -eu

cd /usr/src/noir
yarn workspace @noir-lang/acvm_js test

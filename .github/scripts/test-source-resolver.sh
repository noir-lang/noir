#!/bin/bash
set -eu

cd /usr/src/noir
yarn workspace @noir-lang/source-resolver build
yarn workspace @noir-lang/source-resolver test

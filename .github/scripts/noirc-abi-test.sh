#!/bin/bash
set -eu

cd /usr/src/noir
yarn workspace @noir-lang/noirc_abi test

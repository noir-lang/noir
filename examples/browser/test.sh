#!/bin/bash
set -eu

# This file is used for Noir CI and is not required.
cd $(dirname $0)

nargo compile

# Run the tests
yarn test

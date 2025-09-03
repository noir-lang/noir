#!/bin/bash
set -eu

# This file is used for Noir CI and is not required.
cd $(dirname $0)

# Run the tests (which include building)
yarn test
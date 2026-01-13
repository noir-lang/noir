#!/bin/bash
set -eu

cd $(dirname $0)

# This file is used for Noir CI and is not required.

./transform_and_interpret_ssa.sh

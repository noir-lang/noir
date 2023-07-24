#!/bin/bash
# Used to call this script from a stable path
DIR=$(dirname "$0")
exec node "$DIR/../ts/dest/node/main.js" $@
#!/bin/sh
set -eu

bb() {
  ../dest/node/main.js -v "$@"
}

bb gates
bb prove -o proof
bb write_vk -o vk
bb verify -k vk -p proof
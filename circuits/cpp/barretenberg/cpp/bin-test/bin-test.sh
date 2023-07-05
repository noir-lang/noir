#!/bin/sh
# Script is assumed to be run from
set -eu

bb() {
  ../build/bin/bb "$@" -v
}

bb gates
bb prove -o proof
bb write_vk -o vk
bb verify -k vk -p proof
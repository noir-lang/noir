#!/usr/bin/env bash
set -e

self_path=$(dirname "$(readlink -f "$0")")

package_root=$self_path/../

assert_lt_dir=$package_root/circuits/assert_lt/
nargo --program-dir $assert_lt_dir compile --pedantic-solving

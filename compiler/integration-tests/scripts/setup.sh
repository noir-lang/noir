#!/usr/bin/env bash
set -e

self_path=$(dirname "$(readlink -f "$0")")


$self_path/compile-programs.sh
$self_path/generate-solidity-verifiers.sh
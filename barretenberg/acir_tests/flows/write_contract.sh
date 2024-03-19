#!/bin/sh
set -eu

export TEST_NAME=$(basename $(pwd))

$BIN write_vk  -o vk
$BIN contract -k vk -c $CRS_PATH -b ./target/acir.gz -o $TEST_NAME.sol

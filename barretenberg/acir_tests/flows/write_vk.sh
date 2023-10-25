#!/bin/sh
set -eu

if [ -n "$VERBOSE" ]; then
  $BIN write_vk -v -c $CRS_PATH -o
else
  $BIN write_vk -c $CRS_PATH 
fi
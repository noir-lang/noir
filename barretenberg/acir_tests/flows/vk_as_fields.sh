#!/bin/sh
set -eu

if [ -n "$VERBOSE" ]; then
  $BIN vk_as_fields -v -c $CRS_PATH
else
  $BIN vk_as_fields -c $CRS_PATH 
fi
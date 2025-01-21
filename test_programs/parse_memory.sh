#!/usr/bin/env bash
set -e

# This script accepts a string representing the amount of memory allocated as outputted by `heaptrack`
# and standardizes it to be in terms of megabytes as `heaptrack` will report different units depending on the duration.

DIGITS='([0-9]+(\.[0-9]+)?)'
KILOBYTES_REGEX=^${DIGITS}K$
MEGABYTES_REGEX=^${DIGITS}M$
GIGABYTES_REGEX=^${DIGITS}G$

if [[ $1 =~ $KILOBYTES_REGEX ]]; then
  echo ${BASH_REMATCH[1]} 1000 | awk '{printf "%.3f\n", $1/$2}'
elif [[ $1 =~ $MEGABYTES_REGEX ]]; then
  echo ${BASH_REMATCH[1]} | awk '{printf "%.3f\n", $1}'
elif [[ $1 =~ $GIGABYTES_REGEX ]]; then
  echo ${BASH_REMATCH[1]} 1000 | awk '{printf "%.3f\n", $1*$2}'
else 
  echo "Could not parse memory: unrecognized format" 1>&2
  exit 1
fi

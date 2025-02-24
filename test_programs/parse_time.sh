#!/usr/bin/env bash
set -e

# This script accepts a string representing the time spent within a span as outputted by `tracing`
# and standardizes it to be in terms of seconds as `tracing` will report different units depending on the duration.

DIGITS='([0-9]+(\.[0-9]+)?)'
MICROSECONDS_REGEX=^${DIGITS}µs$
MILLISECONDS_REGEX=^${DIGITS}ms$
SECONDS_REGEX=^${DIGITS}s$

if [[ $1 =~ $MICROSECONDS_REGEX ]]; then
  echo ${BASH_REMATCH[1]} 1000000 | awk '{printf "%.3f\n", $1/$2}'
elif [[ $1 =~ $MILLISECONDS_REGEX ]]; then
  echo ${BASH_REMATCH[1]} 1000 | awk '{printf "%.3f\n", $1/$2}'
elif [[ $1 =~ $SECONDS_REGEX ]]; then
  echo ${BASH_REMATCH[1]} | awk '{printf "%.3f\n", $1}'
else 
  echo "Could not parse time: unrecognized format" 1>&2
  exit 1
fi
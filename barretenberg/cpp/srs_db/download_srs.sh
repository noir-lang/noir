#!/bin/sh
set -eu

AWS_BUCKET=$1
DESTINATION=$2
NUM=${3:-19}
RANGE_START=${4:-}
RANGE_END=${5:-}
APPEND=${6:-"false"}

mkdir -p "$DESTINATION"
cd "$DESTINATION"

if command -v sha256sum > /dev/null; then
  SHASUM=sha256sum
else
  SHASUM="shasum -a 256"
fi

checksum() {
  grep transcript${1}.dat checksums | $SHASUM -c
  return $?
}

download() {
  # Initialize an empty variable for the Range header
  RANGE_HEADER=""

  # If both RANGE_START and RANGE_END are set, add them to the Range header
  if [ -n "$RANGE_START" ] && [ -n "$RANGE_END" ]; then
    RANGE_HEADER="-H Range:bytes=$RANGE_START-$RANGE_END"
  fi

  # Download the file
  if [ "$APPEND" = "true" ]; then
    curl -s $RANGE_HEADER https://aztec-ignition.s3-eu-west-2.amazonaws.com/$AWS_BUCKET/monomial/transcript${1}.dat >> transcript${1}.dat
  else
    curl -s $RANGE_HEADER https://aztec-ignition.s3-eu-west-2.amazonaws.com/$AWS_BUCKET/monomial/transcript${1}.dat > transcript${1}.dat
  fi

}

for TRANSCRIPT in $(seq 0 $NUM); do
  NUM=$(printf %02d $TRANSCRIPT)
  if [ -f checksums  ] && [ -z "$RANGE_START" ] && [ -z "$RANGE_END" ] ; then
    checksum $NUM && continue
    download $NUM
    checksum $NUM || exit 1
  else
    download $NUM
  fi
done

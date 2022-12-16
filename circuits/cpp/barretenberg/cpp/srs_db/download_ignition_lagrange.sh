#!/bin/sh
# Downloads the Lagrange transcripts generated from ignition trusted setup.
#
# To download all transcripts upto 2^{24}.
#  ./download_ignition_lagrange.sh
#
# To download select transcripts upto size 2^m,
#  ./download_ignition_lagrange.sh m
#
# If a checksums file is available, it will be used to validate if a download is required
# and also check the validity of the downloaded transcripts. If not the script downloads
# whatever is requested but does not check the validity of the downloads.
set -e

mkdir -p lagrange
cd lagrange
num_transcripts=${1:-24}
ARGS=$(seq 1 $num_transcripts)

checksum() {
  grep transcript_${1}.dat checksums | sha256sum -c
  return $?
}

download() {
  curl https://aztec-ignition.s3-eu-west-2.amazonaws.com/MAIN%20IGNITION/lagrange/transcript_${1}.dat > transcript_${1}.dat
}

for TRANSCRIPT in $ARGS; do
  NUM=$(printf %2d $((1 << $TRANSCRIPT)))
  if [ -f checksums ]; then
    checksum $NUM && continue
    download $NUM
    checksum $NUM || exit 1
  else
    download $NUM
  fi
done

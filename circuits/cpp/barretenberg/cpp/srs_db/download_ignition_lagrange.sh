#!/bin/sh
# Downloads the Lagrange transcripts generated from ignition trusted setup.
#
# To download all transcripts upto 2^{25}.
#  ./download_ignition_lagrange.sh
#
# To download select transcripts upto size 2^m,
#  ./download_ignition_lagrange.sh m
#
# If a checksums file is available, it will be used to validate if a download is required
# and also check the validity of the downloaded transcripts. If not the script downloads
# whatever is requested but does not check the validity of the downloads.
set -e

cd ignition
mkdir -p lagrange
cd lagrange
num_transcripts=${1:-25}
ARGS=$(seq 1 $num_transcripts)

checksum() {
  grep transcript_${1}.dat checksums | sha256sum -c
  return $?
}

download() {
  curl https://aztec-ignition.s3-eu-west-2.amazonaws.com/MAIN%20IGNITION/lagrange/transcript_${1}.dat > transcript_${1}.dat
}

get_transcript() {
  if [ -f checksums ]; then
    checksum ${1} && return 0
    download ${1}
    checksum ${1} || return 1
  else
    download ${1}
  fi
}

for TRANSCRIPT in $ARGS; do
  NUM=$(printf %2d $((1 << $TRANSCRIPT)))
  get_transcript $NUM

  # We need to handle Lagrange transcripts with > 2^24 points differently 
  # as they're split across multiple transcript files.
  if [ $TRANSCRIPT -gt 24 ]; then
    diff=$(($TRANSCRIPT - 24)) 
    num_sub_transcripts=$((1 << $diff - 1))
    for i in $(seq $num_sub_transcripts)
      do
      NUM_SUB="${NUM}_${i}"
      get_transcript $NUM_SUB
    done
  fi
done

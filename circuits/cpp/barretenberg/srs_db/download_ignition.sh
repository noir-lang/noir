#!/bin/sh
# Downloads the ignition trusted setup transcripts.
#
# To download all transcripts.
#  ./download_ignition.sh
#
# To download select transcripts, e.g. 0 and 1.
#  ./download_ignition.sh 0 1
#
# If a checksums file is available, it will be used to validate if a download is required
# and also check the validity of the downloaded transcripts. If not the script downloads
# whatever is requested but does not check the validity of the downloads.
set -e

mkdir -p ignition
cd ignition
ARGS=$@
[ $# -ne 0 ] || ARGS=$(seq 0 19)

checksum() {
  grep transcript${1}.dat checksums | sha256sum -c
  return $?
}

download() {
  curl https://aztec-ignition.s3-eu-west-2.amazonaws.com/MAIN%20IGNITION/sealed/transcript${1}.dat > transcript${1}.dat
}

for TRANSCRIPT in $ARGS; do
  NUM=$(printf %02d $TRANSCRIPT)
  if [ -f checksums ]; then
    checksum $NUM && continue
    download $NUM
    checksum $NUM || exit 1
  else
    download $NUM
  fi
done
#!/bin/bash
mkdir -p ignition
ARGS=$@
[ $# -ne 0 ] || ARGS=$(seq 0 19)
for TRANSCRIPT in $ARGS; do
  NUM=$(printf %02d $TRANSCRIPT)
  curl https://aztec-ignition.s3-eu-west-2.amazonaws.com/MAIN%20IGNITION/sealed/transcript${NUM}.dat > ignition/transcript${NUM}.dat
done
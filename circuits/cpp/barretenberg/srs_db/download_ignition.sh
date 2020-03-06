#!/bin/bash
mkdir -p ignition
for TRANSCRIPT in $(seq 0 19); do
  NUM=$(printf %02d $TRANSCRIPT)
  curl https://aztec-ignition.s3-eu-west-2.amazonaws.com/MAIN%20IGNITION/sealed/transcript${NUM}.dat > ignition/transcript${NUM}.dat
done
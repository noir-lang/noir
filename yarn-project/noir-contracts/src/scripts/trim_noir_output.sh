#!/bin/bash

# Trim the output of the noir json, removing unneeded fields to make it small enough to be processed by copy_output.ts
# This is a workaround until noir output files get smaller
CONTRACT_NAME=$1

jq -c '.functions[] |= del(.proving_key, .verification_key)' target/main-$CONTRACT_NAME.json > temp.json 
mv temp.json target/main-$CONTRACT_NAME.json
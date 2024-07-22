#!/usr/bin/env bash

PLONK_FLAVOUR="honk"
SRS_PATH="../cpp/srs_db/ignition"
OUTPUT_PATH="./src/honk"

mkdir -p './src/honk/keys'

../cpp/build/bin/honk_solidity_key_gen $PLONK_FLAVOUR add2 $OUTPUT_PATH $SRS_PATH
../cpp/build/bin/honk_solidity_key_gen $PLONK_FLAVOUR blake $OUTPUT_PATH $SRS_PATH
../cpp/build/bin/honk_solidity_key_gen $PLONK_FLAVOUR ecdsa $OUTPUT_PATH $SRS_PATH
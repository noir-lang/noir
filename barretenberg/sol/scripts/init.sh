#!/bin/bash


PLONK_FLAVOUR="ultra"
SRS_PATH="../cpp/srs_db/ignition"
OUTPUT_PATH="./src/ultra"

../cpp/build/bin/solidity_key_gen $PLONK_FLAVOUR add2 $OUTPUT_PATH $SRS_PATH
../cpp/build/bin/solidity_key_gen $PLONK_FLAVOUR blake $OUTPUT_PATH $SRS_PATH
../cpp/build/bin/solidity_key_gen $PLONK_FLAVOUR recursive $OUTPUT_PATH $SRS_PATH
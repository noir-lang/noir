#!/bin/bash

VERSION="v3.0.0-nightly.20250916"

BBUP_PATH=~/.bb/bbup

if ! [ -f $BBUP_PATH ]; then 
    curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/next/barretenberg/bbup/install | bash
fi

$BBUP_PATH -v $VERSION

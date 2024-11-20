#!/bin/bash

VERSION="0.63.0"

BBUP_PATH=~/.bb/bbup

if ! [ -f $BBUP_PATH ]; then 
    curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
fi

$BBUP_PATH -v $VERSION

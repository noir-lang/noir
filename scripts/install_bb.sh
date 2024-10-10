#!/bin/bash

VERSION="0.56.0"

BBUP_PATH=~/.bb/bbup

if ! [ -f $BBUP_PATH ]; then 
    curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/09c3b288eb96d2fe37e7bb6dedb45f9c9f4a4044/barretenberg/cpp/installation/install | bash
fi

$BBUP_PATH -v $VERSION

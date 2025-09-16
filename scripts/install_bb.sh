#!/bin/bash

VERSION="v3.0.0-nightly.20250916"

BBUP_PATH=~/.bb/bbup

if ! [ -f $BBUP_PATH ]; then 
    curl -L https://bbup.aztec.network | bash
fi

$BBUP_PATH -v $VERSION

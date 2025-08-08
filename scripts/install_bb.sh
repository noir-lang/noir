#!/bin/bash

VERSION="0.87.0"

BBUP_PATH=~/.bb/bbup

if ! [ -f $BBUP_PATH ]; then 
    curl -L https://bbup.aztec.network | bash
fi

$BBUP_PATH -v $VERSION

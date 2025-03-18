#!/bin/bash

VERSION="0.77.1"

BBUP_PATH=~/.bb/bbup

if ! [ -f $BBUP_PATH ]; then 
    curl -L https://bbup.aztec.network | bash
fi

$BBUP_PATH -v $VERSION

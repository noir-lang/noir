#!/bin/bash

VERSION="1.0.0-staging.6"

BBUP_PATH=~/.bb/bbup

if ! [ -f $BBUP_PATH ]; then 
    curl -L https://bbup.aztec.network | bash
fi

$BBUP_PATH -v $VERSION

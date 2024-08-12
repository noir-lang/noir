#!/bin/bash

if [ $# -eq 4 ]; then
    COMMAND="$1 $2 $3"
    alias aztec-wallet="${COMMAND}"
    cd $4 # noir-projects/noir-contracts folder
else 
    COMMAND=$1
    alias aztec-wallet="${COMMAND}"
    cd $2 # noir-projects/noir-contracts folder
fi

aztec-wallet () {
    command $COMMAND $@
}

echo "aztec-wallet is $COMMAND"
echo
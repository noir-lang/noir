#!/bin/bash
set -e

LOCATION=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

NOIR_CONTRACTS_PATH=$(realpath ../../../noir-projects/noir-contracts)
USE_DOCKER=$1

export WALLET_DATA_DIRECTORY="${LOCATION}/data"

rm -rf $WALLET_DATA_DIRECTORY
mkdir -p $WALLET_DATA_DIRECTORY

COMMAND="node --no-warnings $(realpath ../dest/bin/index.js)"

if [ "$USE_DOCKER" = "--docker" ]; then
    echo "Using docker"
    COMMAND="aztec-wallet"
fi

cd ./flows

./basic.sh $COMMAND $NOIR_CONTRACTS_PATH
./no_alias.sh $COMMAND $NOIR_CONTRACTS_PATH
./create_account_pay_native.sh $COMMAND $NOIR_CONTRACTS_PATH
./shield_and_transfer.sh $COMMAND $NOIR_CONTRACTS_PATH


#!/bin/bash
set -e

LOCATION=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

NOIR_CONTRACTS_PATH=$(realpath ../../../noir-projects/noir-contracts)

POSITIONAL_ARGS=()

while [[ $# -gt 0 ]]; do
  case $1 in
    -d|--docker)
      USE_DOCKER="1"
      shift
      ;;
    -f|--filter)
      FILTER="$2"
      shift 2
      ;;
    -*|--*)
      echo "Unknown option $1"
      exit 1
      ;;
    *)
      POSITIONAL_ARGS+=("$1") # save positional arg
      shift # past argument
      ;;
  esac
done

set -- "${POSITIONAL_ARGS[@]}" # restore positional parameters

export WALLET_DATA_DIRECTORY="${LOCATION}/data"

rm -rf $WALLET_DATA_DIRECTORY
mkdir -p $WALLET_DATA_DIRECTORY

COMMAND="node --no-warnings $(realpath ../dest/bin/index.js)"

if [ "${USE_DOCKER:-}" = "1" ]; then
    echo "Using docker"
    COMMAND="aztec-wallet"
fi

cd ./flows

for file in $(ls *.sh | grep ${FILTER:-"."}); do
    ./$file $COMMAND $NOIR_CONTRACTS_PATH
done


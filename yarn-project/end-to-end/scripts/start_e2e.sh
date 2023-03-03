#! /bin/bash
set -eu

echo ">> 0"
export NODE_NO_WARNINGS=1
echo ">> 1"
node ${NODE_ARGS-} --openssl-legacy-provider --experimental-vm-modules $(yarn bin jest) --no-cache --runInBand $1
echo ">> 2"
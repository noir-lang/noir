#! /bin/bash
set -eu

export NODE_NO_WARNINGS=1
node ${NODE_ARGS-} --openssl-legacy-provider --experimental-vm-modules $(yarn bin jest) --no-cache --runInBand --passWithNoTests $@

#!/usr/bin/env bash

function usage {
    echo $1
    echo "usage: ./minimize.sh 'error message' (compile|execute) [compile options...] path/to/main.nr [path/to/Prover.toml]"
    exit 1
}

MSG=$1; shift
if [ -z "$MSG" ]; then
    usage "missing error message"
fi

CMD=$1; shift
if [ -z "$CMD" ]; then
    usage "missing command"
fi

# Grab everything until we hit a .nr file. These are our compile options.
OPTIONS=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        *.nr)
            MAIN_PATH=$1
            shift
            break
            ;;
        *)
            OPTIONS+=("$1")
            shift
            ;;
    esac
done

# Build a string from the array of options
OPTIONS="${OPTIONS[*]}"

if [ -z "$MAIN_PATH" ]; then
    usage "missing path to main.nr"
fi
# We need an absolute path for mounting a docker volume
if [[ $MAIN_PATH != /* ]]; then
    MAIN_PATH=$PWD/$MAIN_PATH;
fi
if [ ! -f "$MAIN_PATH" ]; then
    usage "$MAIN_PATH is not a file"
fi

PROVER_PATH=${1:-$(dirname $MAIN_PATH)/../Prover.toml}
if [ ! -f "$PROVER_PATH" ]; then
    if [ "$CMD" == "execute" ]; then
        usage "$PROVER_PATH is not a file"
    else
        # `compile` doesn't need a Prover.toml file, but to keep `docker run`
        # simple we can create an empty one.
        touch $PROVER_PATH
    fi
fi

# Make a copy because the minimizer will modify the file in-place.
cp $MAIN_PATH $MAIN_PATH.bkp

exec docker run --init -it --rm \
    -v "$MAIN_PATH":/noir/main.nr \
    -v "$PROVER_PATH":/noir/Prover.toml \
    -e MSG="$MSG" \
    -e CMD="$CMD" \
    -e OPTIONS="$OPTIONS" \
    noir-minimizer

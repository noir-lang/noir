#!/usr/bin/env bash
set -eu

EXAMPLE_CMD="$0 private_kernel_init"

# First arg is the circuit name.
if [[ $# -eq 0 || ($1 == -* && $1 != "-h") ]]; then
    echo "Please specify the name of the circuit."
    echo "e.g.: $EXAMPLE_CMD"
    exit 1
fi

CIRCUIT_NAME=$1
SERVE=false
PORT=5000
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            echo "Generates a flamegraph for the specified protocol circuit."
            echo ""
            echo "Usage:"
            echo "    $0 <CIRCUIT_NAME>"
            echo ""
            echo "    e.g.: $EXAMPLE_CMD"
            echo ""
            echo "Arguments:"
            echo "    -s    Serve the file over http"
            echo "    -p    Specify custom port. Default: ${PORT}"
            echo ""
            exit 0
            ;;
        -s|--serve)
            SERVE=true
            shift
            ;;
        -p|--port)
            if [[ $# -lt 2 || $2 == -* ]]; then
                echo "Please specify a port number."
                echo "e.g.: $EXAMPLE_CMD -s -p 8080"
                exit 1
            fi
            PORT=$2
            shift 2
            ;;
        *)
            shift
        ;;
    esac
done

# Get the directory of the script.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if the artifact exists.
ARTIFACT="$SCRIPT_DIR/../target/$CIRCUIT_NAME.json"
if [[ ! -f $ARTIFACT ]]; then
    echo "Cannot find artifact: ${ARTIFACT}"
    exit 1
fi

# Build profier if it's not available.
PROFILER="$SCRIPT_DIR/../../../noir/noir-repo/target/release/noir-profiler"
if [ ! -f $PROFILER ]; then
    echo "Profiler not found, building profiler"
    cd "$SCRIPT_DIR/../../../noir/noir-repo/tooling/profiler"
    cargo build --release
    cd "$SCRIPT_DIR"
fi

# We create dest directory and use it as an output for the generated main.svg file.
DEST="$SCRIPT_DIR/../dest"
mkdir -p $DEST

# At last, generate the flamegraph.
$PROFILER gates-flamegraph --artifact-path "${ARTIFACT}" --backend-path "$SCRIPT_DIR/../../../barretenberg/cpp/build/bin/bb"  --output "$DEST" -- -h

# Serve the file over http if -s is set.
if $SERVE; then
    echo "Serving flamegraph at http://0.0.0.0:${PORT}/main.svg"
    python3 -m http.server --directory "$SCRIPT_DIR/../dest" $PORT
fi
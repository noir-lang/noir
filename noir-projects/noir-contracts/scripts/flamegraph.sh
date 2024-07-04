#!/usr/bin/env bash
set -eu

# If first  arg is -h or --help, print usage
if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
    echo "Usage: $0 <contract> <function>"
    echo "e.g.: $0 Token transfer"
    echo "Generates a flamegraph for the given contract and function"
    exit 0
fi

# Get the directory of the script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PROFILER="$SCRIPT_DIR/../../../noir/noir-repo/target/release/noir-profiler"

if [ ! -f $PROFILER ]; then
    echo "Profiler not found, building profiler"
    cd "$SCRIPT_DIR/../../../noir/noir-repo/tooling/profiler"
    cargo build --release
    cd "$SCRIPT_DIR"
fi

# first console arg is contract name in camel case (e.g. TokenBridge)
CONTRACT=$1

# second console arg is the contract function
FUNCTION=$2

function sed_wrapper() {
  if sed --version >/dev/null 2>&1; then
    sed "$@"
  elif gsed --version >/dev/null 2>&1; then
    gsed "$@"
  else
    echo "No suitable sed found"
    echo "You can install gsed with 'brew install gnu-sed'"
    exit 1
  fi
}

# convert contract name to following format: token_bridge_contract-TokenBridge.json
ARTIFACT=$(echo "$CONTRACT" | sed_wrapper -r 's/^([A-Z])/\L\1/; s/([a-z0-9])([A-Z])/\1_\L\2/g')
ARTIFACT_NAME="${ARTIFACT}_contract-${CONTRACT}"

# Extract artifact for the specific function
node "$SCRIPT_DIR/../extractFunctionAsNoirArtifact.js" "$SCRIPT_DIR/../target/${ARTIFACT_NAME}.json" $FUNCTION

FUNCTION_ARTIFACT="${ARTIFACT_NAME}-${FUNCTION}.json"

# We create dest directory and use it as an output for the generated main.svg file
mkdir -p "$SCRIPT_DIR/../dest"

# At last, generate the flamegraph
$PROFILER gates-flamegraph --artifact-path "$SCRIPT_DIR/../target/$FUNCTION_ARTIFACT" --backend-path "$SCRIPT_DIR/../../../barretenberg/cpp/build/bin/bb"  --output "$SCRIPT_DIR/../dest"

# serve the file over http
echo "Serving flamegraph at http://0.0.0.0:8000/main_gates.svg"
python3 -m http.server --directory "$SCRIPT_DIR/../dest" 8000

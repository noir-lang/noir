#!/usr/bin/env bash
set -eu

# TODO(https://github.com/noir-lang/noir/issues/4962): This script is still yet to be integrated with noir-lang/noir-gates-diff
# The script needs some slight updating as `nargo info` expects a complete JSON object, while this script expects a single object field
# representing a list of circuit reports for a program.
# The ACIR tests in barretenberg also expect every target bytecode to have the name `acir.gz` while this script expects the same name of the package
cd noir-protocol-circuits
PROTOCOL_CIRCUITS_DIR=$PWD

BB_BIN=${BB_BIN:-../../barretenberg/cpp/build/bin/bb}

echo "{\"programs\": [" > gates_report.json

# Bound for checking where to place last parentheses
NUM_ARTIFACTS=$(ls -1q "$PROTOCOL_CIRCUITS_DIR/target"/*.json | wc -l)

ITER="1"
for pathname in "$PROTOCOL_CIRCUITS_DIR/target"/*.json; do
    ARTIFACT_NAME=$(basename -s .json "$pathname")

    GATES_INFO=$($BB_BIN gates -h -b "./target/$ARTIFACT_NAME.json")
    MAIN_FUNCTION_INFO=$(echo $GATES_INFO | jq -r '.functions[0] | .name = "main"')
    echo "{\"package_name\": \"$ARTIFACT_NAME\", \"functions\": [$MAIN_FUNCTION_INFO]" >> gates_report.json

    if (($ITER == $NUM_ARTIFACTS)); then
        echo "}" >> gates_report.json
    else
        echo "}, " >> gates_report.json
    fi

    ITER=$(( $ITER + 1 ))
done

echo "]}" >> gates_report.json
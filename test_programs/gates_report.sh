#!/usr/bin/env bash
set -e

BACKEND=${BACKEND:-bb}

# These tests are incompatible with gas reporting
excluded_dirs=(
    "workspace"
    "workspace_default_member"
    "databus"
    "databus_composite_calldata"
    "databus_two_calldata"
    "databus_two_calldata_simple"
    "double_verify_honk_proof"
    "verify_honk_proof"
    "verify_rollup_honk_proof"
    "fold_2_to_17"
    "fold_after_inlined_calls"
    "fold_basic"
    "fold_basic_nested_call"
    "fold_call_witness_condition"
    "fold_complex_outputs"
    "fold_distinct_return"
    "fold_fibonacci"
    "fold_numeric_generic_poseidon"
    "regression_7143"
    "regression_7612"
)

current_dir=$(pwd)
artifacts_path="$current_dir/acir_artifacts"
test_dirs=$(ls $artifacts_path)

echo "{\"programs\": [" > gates_report.json

# Bound for checking where to place last parentheses
NUM_ARTIFACTS=$(ls -1q "$artifacts_path" | wc -l)

ITER="1"
for pathname in $test_dirs; do
    ARTIFACT_NAME=$(basename "$pathname")
    if [[ " ${excluded_dirs[@]} " =~ "$ARTIFACT_NAME" ]]; then
        ITER=$(( $ITER + 1 ))
        continue
    fi
    echo $ARTIFACT_NAME

    GATES_INFO=$($BACKEND gates -b "$artifacts_path/$ARTIFACT_NAME/target/program.json")
    MAIN_FUNCTION_INFO=$(echo $GATES_INFO | jq -r ".functions[0] | {package_name: "\"$ARTIFACT_NAME\"", functions: [{name: \"main\", acir_opcodes, opcodes: .acir_opcodes, circuit_size}], unconstrained_functions: []}")
    echo -n $MAIN_FUNCTION_INFO >> gates_report.json

    if (($ITER == $NUM_ARTIFACTS)); then
        echo "" >> gates_report.json
    else
        echo "," >> gates_report.json
    fi

    ITER=$(( $ITER + 1 ))
done

echo "]}" >> gates_report.json



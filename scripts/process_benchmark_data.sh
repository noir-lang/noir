#!/bin/bash

set -ue

INPUT_DIR=$(realpath $1)

average_times() {
  awk -v RS=" " '
    function parse_time(value) {
      micro_seconds = match($1, /µs$/);
        if (micro_seconds != 0) {
          current_time = substr($1, 0, micro_seconds)
          return current_time / 1000000;
        }
        milli_seconds = match($1, /ms$/);
        if (milli_seconds != 0) {
          current_time = substr($1, 0, milli_seconds)
          return current_time / 1000;
        }
        seconds = match($1, /s$/);
        if (seconds != 0) {
          current_time = substr($1, 0, seconds)
          return current_time;
        }

        printf "Could not parse time: %" $1 > "/dev/stderr"

        printf "ERROR"
        exit 1
    }

    {
      seconds = parse_time($1);
      sum += seconds;
      n++;
    }
    END {
      if (n > 0)
        printf "%.3f\n", sum / n
      else
        printf "%.3f\n", 0
    }' <<<${@}
}

compilation_time() {
  local prefix=${1:-}
  TIMES=($(jq -r '. | select(.target == "nargo::cli" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/${prefix}compilation.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"${prefix}compilation_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

# This measures the time taken for definition collection along with elaboration/type checking. This notably includes
# comptime macro expansion as well, however stops short of the monomorphization and code generation phases.
#
# This is roughly equivalent to the time taken by `nargo check`, ignoring time spent on I/O along with lexing and parsing.
elaboration_time() {
  local prefix=${1:-}
  TIMES=($(jq -r '. | select(.target == "noirc_driver" and .span.name == "check_crate" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/${prefix}compilation.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"${prefix}elaboration_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

execution_time() {
  local prefix=${1:-}
  TIMES=($(jq -r '. | select(.target == "nargo::ops::execute" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/${prefix}execution.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"${prefix}execution_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

artifact_size() {
  ARTIFACT_SIZE=$(wc -c <"$INPUT_DIR/artifact.json" | awk '{printf "%.1f\n", $1/1000}')

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"artifact_size\", value: \""$ARTIFACT_SIZE"\" | tonumber, unit: \"KB\"}" --null-input
}

num_opcodes() {
  num_opcodes=$(noir-inspector info --json "$INPUT_DIR/artifact.json" | jq ".programs[0].functions[0].opcodes")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"num_opcodes\", value: \""$num_opcodes"\" | tonumber, unit: \"opcodes\"}" --null-input
}

brillig_compilation_time() {
  local prefix=${1:-}
  TIMES=($(jq -r '. | select(.target == "nargo::cli" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/${prefix}brillig_compilation.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"${prefix}brillig_compilation_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

brillig_execution_time() {
  local prefix=${1:-}
  TIMES=($(jq -r '. | select(.target == "nargo::ops::execute" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/${prefix}brillig_execution.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"${prefix}brillig_execution_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

brillig_artifact_size() {
  ARTIFACT_SIZE=$(wc -c <"$INPUT_DIR/brillig_artifact.json" | awk '{printf "%.1f\n", $1/1000}')

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"brillig_artifact_size\", value: \""$ARTIFACT_SIZE"\" | tonumber, unit: \"KB\"}" --null-input
}

# The time functions take an optional prefix, `baseline_`, to read the runs of the baseline binary
# instead. Its logs only exist when `gather_benchmark_data.sh` ran with `NARGO_BASELINE`.
baseline_times() {
  local input
  for input in compilation execution brillig_compilation brillig_execution; do
    if [ -f "$INPUT_DIR/baseline_$input.jsonl" ]; then
      case $input in
        compilation)
          compilation_time baseline_
          elaboration_time baseline_
          ;;
        *) ${input}_time baseline_ ;;
      esac
    fi
  done
}

jq --slurp 'reduce .[] as $i ({}; .[$i.metric] = ($i | del(.metric)))' <<< "$(compilation_time)$(elaboration_time)$(execution_time)$(artifact_size)$(num_opcodes)$(brillig_compilation_time)$(brillig_execution_time)$(brillig_artifact_size)$(baseline_times)"

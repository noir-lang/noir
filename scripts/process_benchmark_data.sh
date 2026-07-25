#!/bin/bash

set -ue

INPUT_DIR=$(realpath $1)

# Parses a list of `tracing` duration strings (e.g. "1.2s", "340ms", "57µs") and prints
# each value in seconds on its own line.
parse_times_as_seconds() {
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

    NF {
      printf "%.6f\n", parse_time($1);
    }' <<<${@}
}

# We report the median rather than the mean as wall-clock times on shared CI runners are
# subject to large one-sided spikes which would otherwise drag the average upwards.
#
# Exits with a non-zero status when given no parseable times so that callers skip the metric
# entirely (e.g. execution times for projects marked `cannot_execute`).
median_times() {
  (
    set -o pipefail
    parse_times_as_seconds ${@} | sort -g | awk '
      { values[NR] = $1 }
      END {
        if (NR == 0)
          exit 1
        else if (NR % 2 == 1)
          printf "%.3f\n", values[(NR + 1) / 2]
        else
          printf "%.3f\n", (values[NR / 2] + values[NR / 2 + 1]) / 2
      }'
  )
}

compilation_time() {
  TIMES=($(jq -r '. | select(.target == "nargo::cli" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/compilation.jsonl"))

  MEDIAN_TIME=$(median_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"compilation_time\", value: \""$MEDIAN_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

# This measures the time taken for definition collection along with elaboration/type checking. This notably includes
# comptime macro expansion as well, however stops short of the monomorphization and code generation phases.
#
# This is roughly equivalent to the time taken by `nargo check`, ignoring time spent on I/O along with lexing and parsing.
elaboration_time() {
  TIMES=($(jq -r '. | select(.target == "noirc_driver" and .span.name == "check_crate" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/compilation.jsonl"))

  MEDIAN_TIME=$(median_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"elaboration_time\", value: \""$MEDIAN_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

execution_time() {
  TIMES=($(jq -r '. | select(.target == "nargo::ops::execute" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/execution.jsonl"))

  MEDIAN_TIME=$(median_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"execution_time\", value: \""$MEDIAN_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

artifact_size() {
  ARTIFACT_SIZE=$(wc -c <"$INPUT_DIR/artifact.json" | awk '{printf "%.1f\n", $1/1000}')

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"artifact_size\", value: \""$ARTIFACT_SIZE"\" | tonumber, unit: \"KB\"}" --null-input
}

# Reports the number of instructions executed when compiling the program, as counted by
# cachegrind. Only produced for projects which set `instruction_count: true` in
# `.github/benchmark_projects.yml`.
compilation_instructions() {
  if [ -f "$INPUT_DIR/compilation_instructions.txt" ]; then
    INSTRUCTIONS=$(awk '{printf "%.1f", $1/1000000}' "$INPUT_DIR/compilation_instructions.txt")

    jq -rc "{name: \"$PROJECT_NAME\", metric: \"compilation_instructions\", value: \""$INSTRUCTIONS"\" | tonumber, unit: \"M instrs\"}" --null-input
  fi
}

num_opcodes() {
  num_opcodes=$(noir-inspector info --json "$INPUT_DIR/artifact.json" | jq ".programs[0].functions[0].opcodes")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"num_opcodes\", value: \""$num_opcodes"\" | tonumber, unit: \"opcodes\"}" --null-input
}

brillig_compilation_time() {
  TIMES=($(jq -r '. | select(.target == "nargo::cli" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/brillig_compilation.jsonl"))

  MEDIAN_TIME=$(median_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"brillig_compilation_time\", value: \""$MEDIAN_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

brillig_execution_time() {
  TIMES=($(jq -r '. | select(.target == "nargo::ops::execute" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/brillig_execution.jsonl"))

  MEDIAN_TIME=$(median_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"brillig_execution_time\", value: \""$MEDIAN_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

brillig_artifact_size() {
  ARTIFACT_SIZE=$(wc -c <"$INPUT_DIR/brillig_artifact.json" | awk '{printf "%.1f\n", $1/1000}')

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"brillig_artifact_size\", value: \""$ARTIFACT_SIZE"\" | tonumber, unit: \"KB\"}" --null-input
}

jq --slurp 'reduce .[] as $i ({}; .[$i.metric] = ($i | del(.metric)))' <<< "$(compilation_time)$(compilation_instructions)$(elaboration_time)$(execution_time)$(artifact_size)$(num_opcodes)$(brillig_compilation_time)$(brillig_execution_time)$(brillig_artifact_size)"

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
  TIMES=($(jq -r '. | select(.target == "nargo::cli" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/compilation.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"compilation_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

execution_time() {
  TIMES=($(jq -r '. | select(.target == "nargo::ops::execute" and .fields.message == "close") | .fields."time.busy"' "$INPUT_DIR/execution.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"execution_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

artifact_size() {
  ARTIFACT_SIZE=$(wc -c <"$INPUT_DIR/artifact.json" | awk '{printf "%.1f\n", $1/1000}')

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"artifact_size\", value: \""$ARTIFACT_SIZE"\" | tonumber, unit: \"KB\"}" --null-input
}

num_opcodes() {
  num_opcodes=$(noir-inspector info --json "$INPUT_DIR/artifact.json" | jq ".programs[0].functions[0].opcodes")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"num_opcodes\", value: \""$num_opcodes"\" | tonumber, unit: \"opcodes\"}" --null-input
}

jq --slurp 'reduce .[] as $i ({}; .[$i.metric] = ($i | del(.metric)))' <<< "$(compilation_time)$(execution_time)$(artifact_size)$(num_opcodes)"

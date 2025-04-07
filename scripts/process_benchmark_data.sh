#!/bin/bash

set -ue

INPUT_DIR=$1

OUTPUT_DIR=$(realpath "$(dirname "$0")/output-processed")
mkdir -p $OUTPUT_DIR


average_times() {
  awk -v RS=" " '
    {
      micro_seconds = match($1, /µs$/);
      if (micro_seconds != 0) {
        current_time = substr($1, 0, micro_seconds)
        sum += current_time / 1000000;
        n++;
      }
      milli_seconds = match($1, /ms$/);
      if (milli_seconds != 0) {
        current_time = substr($1, 0, milli_seconds)
        sum += current_time / 1000;
        n++;
      }
      seconds = match($1, /s$/);
      if (seconds != 0) {
        current_time = substr($1, 0, seconds)
        sum += current_time;
        n++;
      } else {
        printf "ERROR"
      }
    }
    END {   
      if (n > 0)
        printf "%.3f\n", sum / n
      else
        printf "%.3f\n", 0
    }' <<<${@}
}

compilation_time() {
  TIMES=($(jq -r '. | select(.target == "nargo::cli" and .fields.message == "close") | .fields."time.busy"' "$PWD/$INPUT_DIR/compilation.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"compilation_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

execution_time() {
  TIMES=($(jq -r '. | select(.target == "nargo::ops::execute" and .fields.message == "close") | .fields."time.busy"' "$PWD/$INPUT_DIR/execution.jsonl"))

  AVG_TIME=$(average_times "${TIMES[@]}")

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"execution_time\", value: \""$AVG_TIME"\" | tonumber, unit: \"s\"}" --null-input
}

artifact_size() {
  ARTIFACT_SIZE=$(wc -c <"$PWD/$INPUT_DIR/artifact.json" | awk '{printf "%.1f\n", $1/1000}')

  jq -rc "{name: \"$PROJECT_NAME\", metric: \"artifact_size\", value: \""$ARTIFACT_SIZE"\" | tonumber, unit: \"KB\"}" --null-input
}

jq --slurp 'reduce .[] as $i ({}; .[$i.metric] = ($i | del(.metric)))' <<< "$(compilation_time)$(execution_time)$(artifact_size)"

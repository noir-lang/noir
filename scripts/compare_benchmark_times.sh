#!/bin/bash

# Prints markdown tables comparing the times of the baseline binary with those of the binary under
# test, one table per metric. Each report in the directory is a JSON object mapping a metric, such
# as `compilation_time`, to `{ name, value, unit }`, with the baseline's time under the same metric
# prefixed by `baseline_`. Both times must come from the same job so they were measured on the same
# machine.
#
# A metric's table is printed if some project's ratio differs from 1 by at least `THRESHOLD`
# (a fraction, 0.1 by default), or for every metric when `SHOW_ALL` is `true`. Ratios of times that
# are both below `MIN_TIME` seconds (0.1 by default) are never flagged: at that scale they mostly
# measure timer resolution and scheduling.

set -ue

REPORTS_DIR=$(realpath $1)
THRESHOLD=${THRESHOLD:-0.1}
SHOW_ALL=${SHOW_ALL:-false}
MIN_TIME=${MIN_TIME:-0.1}

shopt -s nullglob
REPORTS=("$REPORTS_DIR"/*)
if [ ${#REPORTS[@]} -eq 0 ]; then
  exit 0
fi

jq --slurp -r --argjson threshold "$THRESHOLD" --argjson show_all "$SHOW_ALL" --argjson min_time "$MIN_TIME" '
  def metrics: [
    ["compilation_time", "Compilation time"],
    ["elaboration_time", "Elaboration time"],
    ["execution_time", "Execution time"],
    ["brillig_compilation_time", "Brillig compilation time"],
    ["brillig_execution_time", "Brillig execution time"],
    ["test_suite_duration", "Test suite duration"]
  ];
  def ratio($base; $new): if $base > 0 then $new / $base else 1 end;
  def changed: ([.base, .new] | max) >= $min_time and ((.ratio - 1 | fabs) >= $threshold);
  def fmt: (.ratio * 100 | round / 100 | tostring) as $s
    | if changed then "**\($s)**" else $s end;

  . as $reports
  | [
      metrics[] as [$metric, $title]
      | [
          $reports[]
          | select(.[$metric] and .["baseline_" + $metric])
          | {
              name: .[$metric].name,
              unit: .[$metric].unit,
              base: .["baseline_" + $metric].value,
              new: .[$metric].value,
            }
          | .ratio = ratio(.base; .new)
        ]
      | sort_by(.name)
      | select(length > 0 and ($show_all or any(.[]; changed)))
      | [
          "## \($title)",
          "",
          "| Project | Base | PR | Ratio |",
          "|-|-|-|-|",
          (.[] | "| `\(.name)` | \(.base) \(.unit) | \(.new) \(.unit) | \(fmt) |"),
          ""
        ]
      | join("\n")
    ]
  | select(length > 0)
  | [
      "# Timings on the same machine",
      "",
      "Each project was run with both the base branch'"'"'s `nargo` and this PR'"'"'s, in the same job.",
      "Ratios at least \($threshold * 100 | round)% away from 1 are in bold, unless both times are under \($min_time) s.",
      ""
    ] + .
  | join("\n")
' "${REPORTS[@]}"

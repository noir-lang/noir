#!/bin/bash

# Prints a markdown table comparing the compilation and elaboration times of the baseline binary
# with those of the binary under test, as recorded by `process_benchmark_data.sh` when both were
# run in the same job.
#
# Prints nothing unless some project's ratio differs from 1 by at least `THRESHOLD` (a fraction,
# 0.1 by default), or `SHOW_ALL` is `true`.

set -ue

REPORTS_DIR=$(realpath $1)
THRESHOLD=${THRESHOLD:-0.1}
SHOW_ALL=${SHOW_ALL:-false}

shopt -s nullglob
REPORTS=("$REPORTS_DIR"/*)
if [ ${#REPORTS[@]} -eq 0 ]; then
  exit 0
fi

jq --slurp -r --argjson threshold "$THRESHOLD" --argjson show_all "$SHOW_ALL" '
  def ratio($base; $new): if $base > 0 then $new / $base else 1 end;
  def changed($r): ($r - 1 | fabs) >= $threshold;
  def fmt($r): ($r * 100 | round / 100 | tostring) as $s
    | if changed($r) then "**\($s)**" else $s end;

  map(select(.baseline_compilation_time and .baseline_elaboration_time)
    | {
        name: .compilation_time.name,
        base_c: .baseline_compilation_time.value,
        new_c: .compilation_time.value,
        base_e: .baseline_elaboration_time.value,
        new_e: .elaboration_time.value,
      }
    | .ratio_c = ratio(.base_c; .new_c)
    | .ratio_e = ratio(.base_e; .new_e))
  | sort_by(.name)
  | if length > 0 and ($show_all or any(.[]; changed(.ratio_c) or changed(.ratio_e))) then
      [
        "# Compilation time on the same machine",
        "",
        "Each project was compiled with the base branch'"'"'s `nargo` and this PR'"'"'s, alternating, in the same job.",
        "Ratios at least \($threshold * 100 | round)% away from 1 are in bold.",
        "",
        "| Project | Compilation (base) | Compilation (PR) | Ratio | Elaboration (base) | Elaboration (PR) | Ratio |",
        "|-|-|-|-|-|-|-|",
        (.[] | "| `\(.name)` | \(.base_c) s | \(.new_c) s | \(fmt(.ratio_c)) | \(.base_e) s | \(.new_e) s | \(fmt(.ratio_e)) |")
      ] | join("\n")
    else
      empty
    end
' "${REPORTS[@]}"

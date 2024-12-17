#!/bin/bash
set -eu

combined_reports='{"programs": []}'

# Iterate over each report and merge them
for report in ./reports/*; do
  # The report is saved under ./$REPORT_NAME_{ matrix_report }/$REPORT_NAME_{ matrix_report }.json
  FILE_PATH=$(echo $(ls $report)) 

  # Extract the programs array from each report and merge it
  combined_reports=$(jq --argjson COMBINED_REPORTS "$combined_reports" '.programs += $COMBINED_REPORTS.programs' <<< "$(cat "$report/$FILE_PATH")")
done

echo "$combined_reports"


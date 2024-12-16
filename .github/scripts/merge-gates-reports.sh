#!/bin/bash
set -eu

REPORT_NAME=$1
NAME_PLURAL=""$REPORT_NAME"s"

combined_reports='{"programs": []}'

# Iterate over each report and merge them
for report in ./reports/*; do
  # The report is saved under ./$REPORT_NAME_{ matrix_report }/$REPORT_NAME_{ matrix_report }.json
  FILE_PATH=$(echo $(ls $report)) 

  # Extract the $NAME_PLURAL array from each report and merge it
  combined_reports=$(jq --argjson COMBINED_REPORTS "$combined_reports" '.programs += $COMBINED_REPORTS.programs' <<< "$(cat "$report/$FILE_PATH")")
done

# Wrap the merged memory reports into a new object as to keep the $NAME_PLURAL key

echo "$combined_reports"


#!/bin/bash
set -eu

echo "Merging reports"

REPORT_NAME=$1

combined_reports="[]"

# Iterate over each report and merge them
for report in ./reports/*; do
  # The report is saved under ./memory_report_{ matrix_report }/memory_report_{ matrix_report }.json
  REPORT_NAME=$(echo $(ls $report)) 
  # Extract the 'memory_reports' array from each report and merge it
  combined_reports=$(jq '[."'$REPORT_NAME'"[]] + '"$combined_reports" <<< "$(cat "$report/$REPORT_NAME")")
done

combined_reports=$(jq '[."'$REPORT_NAME'"[]] + '"$combined_reports" <<< "$(cat ./$REPORT_NAME.json)")

# Wrap the merged memory reports into a new object as to keep the 'memory_reports' key
final_report="{\"$REPORT_NAME\": $combined_reports}"

echo "$final_report" > $REPORT_NAME.json

cat $REPORT_NAME.json
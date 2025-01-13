#!/bin/bash
set -eu

echo "Merging reports"

REPORT_NAME=$1
NAME_PLURAL=""$REPORT_NAME"s"

combined_reports="[]"

# Iterate over each report and merge them
for report in ./reports/*; do
  # The report is saved under ./$REPORT_NAME_{ matrix_report }/$REPORT_NAME_{ matrix_report }.json
  FILE_PATH=$(echo $(ls $report)) 

  # Extract the $NAME_PLURAL array from each report and merge it
  combined_reports=$(jq '[."'"$NAME_PLURAL"'"[]] + '"$combined_reports" <<< "$(cat "$report/$FILE_PATH")")
done

combined_reports=$(jq '[."'$NAME_PLURAL'"[]] + '"$combined_reports" <<< "$(cat ./$REPORT_NAME.json)")

# Wrap the merged memory reports into a new object as to keep the $NAME_PLURAL key
final_report="{\"$NAME_PLURAL\": $combined_reports}"

echo "$final_report" > $REPORT_NAME.json

cat $REPORT_NAME.json
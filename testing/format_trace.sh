#!/usr/bin/env bash
# Run with ./format_trace.sh <file-to-format.json> <output-file.json>
# where the second argument is optional. If it's not stated the output will be saved at <file-to-format>+modified.json
# where '+' is string concatination.

set -euo pipefail

if [ $# -eq 0 ]; then
  echo "Read the source"
  exit 1
fi

filename="$1"
begin_target_line="  {"
end_target_line=" },"

if [ $# -eq 1 ]; then
  destination_name="${filename%%.*}_modified.json"
else 
  destination_name="$2"
fi

if [ ! -f "$filename" ]; then
  echo "File not found!"
  exit 1
fi

jq . $filename > "${filename%%.*}_modtest.json"

filename="${filename%%.*}_modtest.json"

temp_file=$(mktemp)
sed 's|"Path": ".*/src/\([^"]*\)"|"Path": "<relative-to-this>/src/\1"|' "$filename" > "$temp_file"
mv "$temp_file" "$filename"

temp_file=$(mktemp)

previous_line=""
while IFS= read -r line; do
  if [ "$line" != "$begin_target_line" ] && [ "$line" != "[" ] && [ "$line" != "]" ] && [ -n "$previous_line" ]; then
    line=$(echo "$line" | sed 's/^[ \t]*//')
    line=" $line"
  fi

  if [ -n "$previous_line" ]; then
    if [ "$previous_line" != "$end_target_line" ] && [ "$previous_line" != "[" ] && [ "$previous_line" != "]" ]; then
        echo -n "$previous_line" >> "$temp_file"
    else 
        echo "$previous_line" >> "$temp_file"
    fi
  fi

  previous_line="$line"
done < "$filename"

if [ -n "$previous_line" ]; then
  previous_line=$'\n'"$previous_line"
  echo "${previous_line}" >> "$temp_file"
fi
rm "$filename"
mv "$temp_file" "$destination_name"

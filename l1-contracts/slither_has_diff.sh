#!/bin/bash

FILE="slither_output.md"

DIFF_OUTPUT=$(git diff -- "$FILE")

if [ -z "$DIFF_OUTPUT" ]; then
    echo "No difference found."
else
    echo "Difference found!"
    exit 1 
fi

#!/bin/bash
set -eu

cd $(dirname $0)

NARGO=${NARGO:-nargo}

"$NARGO" test --coverage

GENERATED=target/coverage/lcov.info
REFERENCE=lcov.info

if [ -f "$REFERENCE" ]; then
    echo "Verifying coverage report..."
    diff "$REFERENCE" "$GENERATED"
    echo "Coverage report matches expected."
    rm "$GENERATED"
else
    echo "No reference coverage report found. Saving generated report..."
    mv "$GENERATED" "$REFERENCE"
    echo "Saved to $REFERENCE. Commit this file to check it into Git."
fi

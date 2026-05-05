#!/bin/bash
set -eu

cd $(dirname $0)

NARGO=${NARGO:-nargo}

"$NARGO" test --coverage

GENERATED=target/coverage/lcov.info
REFERENCE=lcov.info

# Strip the machine-specific prefix from SF: records so the file is
# portable and can be checked into Git.  Everything before the last
# occurrence of "examples/" is removed (greedy .* backtracks to the
# rightmost match), leaving paths of the form "SF:examples/coverage/src/...".
sed -i.bak 's|SF:.*/examples/|SF:examples/|' "$GENERATED" && rm -f "${GENERATED}.bak"

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

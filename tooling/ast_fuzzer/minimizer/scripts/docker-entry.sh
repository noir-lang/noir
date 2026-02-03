#!/usr/bin/env bash

set -e

# We assume that there is a `main.nr` and a `Prover.toml` file mounted in the working directory,
# which `cvise` is going to copy to a temporary directory, each time with some new minimization.

# Create a check script which creates a new project with the contents we want to test,
# run a `nargo` command, and checks the presence of the error message in the output.
# If it's not present, it means the latest reduction step was not interesting.
# We could have the script read env vars on the fly instead of splicing them in verbatim,
# but this is perhaps closer to how `cvise` wants an parameterless script.

cat > check.sh <<EOF
nargo new test_project
cp main.nr test_project/src
cp /noir/Prover.toml test_project
cd test_project
nargo $CMD $OPTIONS 2>&1 | grep "$MSG"
EOF

chmod +x check.sh

cvise --not-c ./check.sh main.nr
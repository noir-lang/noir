#!/usr/bin/env bash
# This is a wrapper script for nargo.
# Pass any args that you'd normally pass to nargo.
# If the first arg is "compile",
# run nargo and then transpile any created artifacts.
#
# Usage: compile_then_transpile.sh [nargo args]
set -eu

NARGO=${NARGO:-nargo}
TRANSPILER=${TRANSPILER:-avm-transpiler}

if [ "${1:-}" != "compile" ]; then
  # if not compiling, just pass through to nargo verbatim
  $NARGO $@
  exit $?
fi
shift # remove the compile arg so we can inject --show-artifact-paths

# Forward all arguments to nargo, tee output to console
artifacts_to_transpile=$($NARGO compile --show-artifact-paths $@ | tee /dev/tty | grep -oP 'Saved contract artifact to: \K.*')

# NOTE: the output that is teed to /dev/tty will normally not be redirectable by the caller.
# If the script is run via docker, however, the user will see this output on stdout and will be able to redirect.

# Transpile each artifact
for artifact in "$artifacts_to_transpile"; do
  # transpiler input and output files are the same (modify in-place)
  $TRANSPILER "$artifact" "$artifact"
done

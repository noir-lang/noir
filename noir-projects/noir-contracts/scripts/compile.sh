#!/usr/bin/env bash
set -euo pipefail

echo "Compiling contracts..."
../../noir/noir-repo/target/release/nargo compile --silence-warnings

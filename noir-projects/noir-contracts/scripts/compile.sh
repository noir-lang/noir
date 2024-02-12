#!/usr/bin/env bash
set -euo pipefail

echo "Compiling contracts..."
../../noir/target/release/nargo compile --silence-warnings

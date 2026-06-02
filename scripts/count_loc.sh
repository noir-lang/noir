#!/usr/bin/env bash
set -eu

# Run relative to repo root
cd $(dirname "$0")/../

if ! command -v "tokei" >/dev/null 2>&1; then
    echo "Error: tokei is required but not installed." >&2
    echo "Error: Run \`cargo install --git https://github.com/TomAFrench/tokei --branch tf/add-noir-support tokei\`" >&2

    exit 1
fi

echo ""
echo "Total:"

tokei ./ --sort code

echo ""
echo "ACIR/ACVM:"
tokei ./acvm-repo --sort code

echo ""
echo "Compiler:"
tokei ./compiler --sort code

echo ""
echo "Tooling:"
tokei ./tooling --sort code

echo ""
echo "Standard Library:"
tokei ./noir_stdlib --sort code

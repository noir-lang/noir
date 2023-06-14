#!/usr/bin/env bash

# Extract version from Cargo.toml using toml2json
PACKAGE_VERSION=$(toml2json < Cargo.toml | jq -r .workspace.package.version)
if [ -z "$PACKAGE_VERSION" ]; then
    echo "Could not extract version from Cargo.toml"
    exit 1
fi
PACKAGE_VERSION+=$VERSION_APPENDIX

echo PACKAGE_VERSION=$PACKAGE_VERSION

mkdir -p $out
cp README.md $out/
cp -r ./pkg/* $out/
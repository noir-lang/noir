#!/usr/bin/env bash
export self_path=$(dirname "$(readlink -f "$0")")

export out_path=$out/noir_wasm

SEMVER_SUFFIX=""

# e.g. v1.0.0-alpha
if [[ -n "${SEMVER_PRE_RELEASE}" ]]; then
  SEMVER_SUFFIX="$SEMVER_SUFFIX-$SEMVER_PRE_RELEASE"
fi

# e.g. v1.0.0-alpha+sha.abcdef
if [[ -n "${SEMVER_BUILD_META}" ]]; then
  SEMVER_SUFFIX="$SEMVER_SUFFIX+$SEMVER_BUILD_META"
fi

mkdir -p $out_path
cp $self_path/README.md $out_path/
jq --arg semver_suffix "$SEMVER_SUFFIX" '.version = .version + $semver_suffix' $self_path/package.json > $out_path/package.json
cp -r $self_path/nodejs $out_path/
cp -r $self_path/web $out_path/

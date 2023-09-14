#!/usr/bin/env bash

set -x

export self_path=$(dirname "$(readlink -f "$0")")

mkdir -p $out
cp $self_path/README.md $out/
cp -r $self_path/nodejs $out/
cp -r $self_path/web $out/

# The main package.json contains several keys which are incorrect/unwanted when distributing.
cat $self_path/package.json \
| jq 'del(.private, .devDependencies, .scripts, .packageManager)' \
> $out/package.json

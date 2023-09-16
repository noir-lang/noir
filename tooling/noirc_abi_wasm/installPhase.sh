#!/usr/bin/env bash
export self_path=$(dirname "$(readlink -f "$0")")

mkdir -p $out
cp $self_path/README.md $out/
cp $self_path/package.json $out/
cp -r $self_path/nodejs $out/
cp -r $self_path/web $out/

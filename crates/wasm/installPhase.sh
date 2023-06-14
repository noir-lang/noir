#!/usr/bin/env bash

mkdir -p $out
cp README.md $out/
cp package.json $out/
cp -r ./pkg/* $out/
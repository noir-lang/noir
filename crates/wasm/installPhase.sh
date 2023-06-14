#!/usr/bin/env bash

mkdir -p $out
cp README.md $out/
cp package.json $out/

echo "-------------"
ls -la 
echo "-------------"
ls -la $out/
echo "-------------"

cp -r ./pkg/* $out/
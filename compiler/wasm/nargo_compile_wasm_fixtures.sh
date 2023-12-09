#!/bin/bash

cd ./fixtures
for dir in $(ls -d */); do
    pushd $dir/noir-script
    nargo compile
    popd
done

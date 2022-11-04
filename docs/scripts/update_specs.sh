#/bin/bash

git clone "https://github.com/AztecProtocol/aztec-connect.git" "./tmp/"
cp -R ./tmp/specs/aztec-connect/src/ ./docs/spec
rm -rf ./tmp
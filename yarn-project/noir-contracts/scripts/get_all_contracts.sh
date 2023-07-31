#!/bin/bash
# Utility to get the names of all contracts
echo $(ls -d src/contracts/*_contract/Nargo.toml | sed -r "s/src\\/contracts\\/(.+)_contract\\/Nargo.toml/\\1/")
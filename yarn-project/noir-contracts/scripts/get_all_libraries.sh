#!/usr/bin/env bash
# Utility to get the names of all noir libraries located in ../aztec-nr
echo $(ls -d ../aztec-nr/*/Nargo.toml | sed -r "s/..\\/aztec-nr\\/(.+)\\/Nargo.toml/\\1/")
#!/bin/bash
# Utility to get the names of all noir libaries located in ../aztec-nr
echo $(ls -d ../aztec-nr/*/Nargo.toml | sed -r "s/..\\/aztec-nr\\/(.+)\\/Nargo.toml/\\1/")
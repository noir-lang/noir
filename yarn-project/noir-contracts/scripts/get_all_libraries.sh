#!/bin/bash
# Utility to get the names of all noir libaries located in ../noir-libs
echo $(ls -d ../noir-libs/*/Nargo.toml | sed -r "s/..\\/noir-libs\\/(.+)\\/Nargo.toml/\\1/")
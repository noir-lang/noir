#!/bin/bash

# Rebuild
./scripts/compile_avm.sh

# Format generated folders
git add **/generated/*
./format.sh staged

# Build vm tests
cmake --build --preset clang16 --target vm_tests
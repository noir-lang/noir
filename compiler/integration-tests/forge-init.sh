#!/bin/bash

cd "$(dirname "$0")"

# Create forge project
forge init --no-git --no-commit --force foundry-project

# Remove default .sol files
rm -rf ./foundry-project/*/**.sol

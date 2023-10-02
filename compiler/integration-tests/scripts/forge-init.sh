#!/bin/bash

self_path=$(dirname "$(readlink -f "$0")")

project_path=$self_path/../foundry-project

# Create forge project
forge init --no-git --no-commit --no-deps --force $project_path

# Remove unwanted files
rm -rf $project_path/script
rm -rf $project_path/src/*.sol
rm -rf $project_path/test

#!/bin/bash

self_path=$(dirname "$(readlink -f "$0")")

bash $self_path/forge-init.sh
bash $self_path/codegen-verifiers.sh
bash $self_path/deploy-verifiers.sh

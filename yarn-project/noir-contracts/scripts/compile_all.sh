#!/usr/bin/env bash
echo "Compiling all contracts"
./scripts/compile.sh $(./scripts/get_all_contracts.sh)

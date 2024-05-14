#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

REFERENCE_DIR="./processed-docs/reference"
NARGO_REFERENCE="$REFERENCE_DIR/nargo_commands.md"
rm -f $NARGO_REFERENCE
mkdir -p $REFERENCE_DIR

echo "---
title: Nargo
description:
  Noir CLI Commands for Noir Prover and Verifier to create, execute, prove and verify programs,
  generate Solidity verifier smart contract and compile into JSON file containing ACIR
  representation and ABI of circuit.
keywords:
  [
    Nargo,
    Noir CLI,
    Noir Prover,
    Noir Verifier,
    generate Solidity verifier,
    compile JSON file,
    ACIR representation,
    ABI of circuit,
    TypeScript,
  ]
sidebar_position: 0
---
" > $NARGO_REFERENCE

cargo run --bin nargo -F codegen-docs -- info >> $NARGO_REFERENCE

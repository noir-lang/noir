#!/usr/bin/env bash
set -eu

TRANSPILER=${TRANSPILER:-../../avm-transpiler/target/release/avm-transpiler}
ls target/avm_*.json | parallel "$TRANSPILER {} {}"
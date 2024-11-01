#! /bin/sh

llvm-cov show -Xdemangler=rustfilt target/debug/nargo -instr-profile=noir_tests.profdata -show-line-counts-or-regions -show-instantiations

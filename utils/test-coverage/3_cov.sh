#! /bin/sh

# shows detailes line by line coverage in the source code (very long output):
llvm-cov show -Xdemangler=rustfilt target/debug/nargo -instr-profile=noir_tests.profdata -show-line-counts-or-regions -show-instantiations

# shows a summary of the coverage in a given cargo package (compiler/noirc_frontend in this example):
llvm-cov report --use-color --instr-profile=./noir_tests.profdata --object target/debug/nargo --sources `find compiler/noirc_frontend -name "*.rs"`

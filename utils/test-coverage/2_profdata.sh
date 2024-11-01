#! /bin/sh

llvm-profdata merge -sparse `find . -name "*.profraw"` -o noir_tests.profdata

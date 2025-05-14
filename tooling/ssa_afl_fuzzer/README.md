# SSA AFL Fuzzer

## Running the fuzzer

```bash
# need to rebuild w/ AFL before fuzzing:
❯ nohup cargo afl fuzz -i inputs -o outputs2 -M fuzzer01 ../../target/debug/ssa_afl_fuzzer &> fuzzer01.log &; tail -f fuzzer01.log

# first worker needs "-M"
❯ nohup cargo afl fuzz -i inputs -o outputs2 -M fuzzer01 ../../target/debug/ssa_afl_fuzzer &> fuzzer01.log &; tail -f fuzzer01.log
..

# other workers need "-S"
❯ nohup cargo afl fuzz -i inputs -o outputs2 -S fuzzer10 ../../target/debug/ssa_afl_fuzzer &> fuzzer10.log &; tail -f fuzzer10.log
```


## To reproduce a crash

```bash
❯ cargo afl run ../../target/debug/ssa_afl_fuzzer < outputs/fuzzer01/crashes/id:000000,sig:06,src:001205,time:60283,execs:196992,op:quick,pos:1261
```

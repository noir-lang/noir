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

## Scripts

In short:
- `keep_inputs_upto_2kb`: AFL appears to work best on inputs up to ~1-2KB
- `make_combined_inputs`: The current fuzzer lets AFL choose which passes to run, so per-pass inputs may be combined
- `collect_unique_crashes`: Find output-unique crashes
- `unique_crashes_representative_inputs`: Collect location-unique crashes and their outputs

### `keep_inputs_upto_2kb.rb`

Only retain inputs `<= 2KB`.

Note: currently requires editing the target directory of inputs

### `make_combined_inputs.rb`

Combine inputs from different target passes into `./combined_inputs`

### `collect_outputs.rb`

Note: it may be better to run `collect_unique_crashes.rb` directly.

Collect outputs from different fuzzers and runs into `./collected_outputs`.

### `collect_unique_crashes.rb`

Collect output-unique crashes into `./unique_crashes`

Note: this will emit warnings if any of the crashes fail to be reproduced.
If many crashes fail to be reproduced, the `ssa_afl_fuzzer` binary may be out of
date.

### `unique_crashes_panic_locations.rb`

Summarize results from `collect_unique_crashes.rb`, only showing the location of
the crash.

### `unique_crashes_representative_inputs.rb`

Find representative inputs (along with their outputs) from `./unique_crashes`
and store them in `./unique_crash_location_representatives`.

### `force_quit_fuzzing_job.rb`

Force-quit the fuzzing runners and targets.

## To reproduce a crash

```bash
❯ cargo afl run ../../target/debug/ssa_afl_fuzzer < outputs/fuzzer01/crashes/id:000000,sig:06,src:001205,time:60283,execs:196992,op:quick,pos:1261
```

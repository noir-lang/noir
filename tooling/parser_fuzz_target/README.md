
## Running

```bash
AFL_AUTORESUME=1 ccargo afl fuzz -M main-$HOSTNAME -i in -o out ../../target/debug/parser_fuzz_target
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser_fuzz_target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser_fuzz_target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser_fuzz_target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser_fuzz_target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser_fuzz_target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser_fuzz_target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser_fuzz_target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser_fuzz_target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer10 -i in -o out ../../target/debug/parser_fuzz_target 
```

## Results

### acvm inputs

```bash
 AFL ++4.10c {acvm_fuzzer0} (...rget/debug/acvm_inputs_fuzz_target) [explore]
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 2 hrs, 43 min, 51 sec     │  cycles done : 376   │
│   last new find : 0 days, 0 hrs, 5 min, 5 sec       │ corpus count : 5399  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 3972*10 (73.6%)    │    map density : 0.41% / 1.92%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.31 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 7               │ favored items : 604 (11.19%)        │
│ stage execs : 128/225 (56.89%)       │  new edges on : 911 (16.87%)        │
│ total execs : 238M                   │ total crashes : 0 (0 saved)         │
│  exec speed : 19.5k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : disabled (default, enable with -D)   │    levels : 33        │
│  byte flips : disabled (default, enable with -D)   │   pending : 1         │
│ arithmetics : disabled (default, enable with -D)   │  pend fav : 0         │
│  known ints : disabled (default, enable with -D)   │ own finds : 4566      │
│  dictionary : n/a                                  │  imported : 0         │
│havoc/splice : 2877/85.1M, 1324/152M                │ stability : 98.85%    │
│py/custom/rq : unused, unused, 121/507k, 244/93.3k  ├───────────────────────┘
│    trim/eff : disabled, disabled                   │             [cpu: 62%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++
[+] We're done here. Have a nice day!
```

```bash
 AFL ++4.10c {lexer_fuzzer0} (...../target/debug/lexer_fuzz_target) [explore]
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 20 hrs, 16 min, 26 sec    │  cycles done : 96    │
│   last new find : 0 days, 17 hrs, 3 min, 7 sec      │ corpus count : 2559  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1472.75 (57.5%)    │    map density : 0.18% / 0.61%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 7.44 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 163 (6.37%)         │
│ stage execs : 209k/1.97M (10.65%)    │  new edges on : 288 (11.25%)        │
│ total execs : 46.5M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 13.08/sec (zzzz...)    │  total tmouts : 63 (0 saved)        │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : disabled (default, enable with -D)   │    levels : 17        │
│  byte flips : disabled (default, enable with -D)   │   pending : 2         │
│ arithmetics : disabled (default, enable with -D)   │  pend fav : 0         │
│  known ints : disabled (default, enable with -D)   │ own finds : 2068      │
│  dictionary : n/a                                  │  imported : 0         │
│havoc/splice : 1086/16.1M, 845/27.9M                │ stability : 100.00%   │
│py/custom/rq : unused, unused, 82/1.94M, 54/356k    ├───────────────────────┘
│    trim/eff : disabled, disabled                   │             [cpu: 61%]
└─ strategy: exploit ────────── state: finished... ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++
[+] We're done here. Have a nice day!
```


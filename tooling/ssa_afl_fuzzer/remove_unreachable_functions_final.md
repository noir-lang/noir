
Worker 1:

```bash
     AFL ++4.21c {fuzzer01} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 8 min, 44 sec      │  cycles done : 486   │
│   last new find : 0 days, 0 hrs, 9 min, 24 sec      │ corpus count : 1573  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1247*20 (79.3%)    │    map density : 0.13% / 0.42%      │                                                                                                                                          
│  runs timed out : 0 (0.00%)          │ count coverage : 2.38 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 12              │ favored items : 124 (7.88%)         │
│ stage execs : 3/37 (8.11%)           │  new edges on : 222 (14.11%)        │
│ total execs : 79.3M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 22.7k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/19.2k, 0/19.2k, 0/19.2k            │    levels : 13        │
│  byte flips : 0/2406, 0/2403, 0/2397               │   pending : 0         │
│ arithmetics : 1/168k, 0/336k, 0/335k               │  pend fav : 0         │
│  known ints : 0/21.6k, 0/91.3k, 0/134k             │ own finds : 427       │
│  dictionary : 0/0, 0/0, 0/24.1k, 0/24.1k           │  imported : 237       │
│havoc/splice : 288/28.3M, 81/50.6M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 54/353k, 0/0         ├───────────────────────┘
│    trim/eff : disabled, 99.88%                     │          [cpu000: 15%]
└─ strategy: explore ────────── state: in progress ──┘^C[110;10u

+++ Testing aborted by user +++
```

Worker 2:

```bash
     AFL ++4.21c {fuzzer02} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐                                                                                                                                          
│        run time : 0 days, 1 hrs, 8 min, 30 sec      │  cycles done : 42    │
│   last new find : 0 days, 0 hrs, 9 min, 51 sec      │ corpus count : 1584  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1269.593 (80.1%)   │    map density : 0.08% / 0.42%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.38 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 9               │ favored items : 128 (8.08%)         │
│ stage execs : 72/75 (96.00%)         │  new edges on : 225 (14.20%)        │
│ total execs : 68.0M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 19.9k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/992, 0/990, 0/986                  │    levels : 10        │
│  byte flips : 0/124, 0/122, 0/118                  │   pending : 0         │
│ arithmetics : 0/8652, 0/16.8k, 0/16.2k             │  pend fav : 0         │
│  known ints : 0/1109, 0/4616, 0/6588               │ own finds : 429       │
│  dictionary : 0/0, 0/0, 0/2804, 0/2872             │  imported : 246       │
│havoc/splice : 347/24.6M, 71/43.0M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 9/292k, 0/0          ├───────────────────────┘
│    trim/eff : 38.08%/78.3k, 98.39%                 │          [cpu001:  9%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++
```

Worker 3:

```bash
     AFL ++4.21c {fuzzer03} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 8 min, 23 sec      │  cycles done : 41    │
│   last new find : 0 days, 0 hrs, 9 min, 7 sec       │ corpus count : 1576  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1448.323 (91.9%)   │    map density : 0.11% / 0.42%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.38 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 2               │ favored items : 135 (8.57%)         │
│ stage execs : 27/37 (72.97%)         │  new edges on : 225 (14.28%)        │
│ total execs : 65.0M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 18.6k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/904, 0/901, 0/895                  │    levels : 10        │
│  byte flips : 0/113, 0/110, 0/104                  │   pending : 0         │
│ arithmetics : 0/7868, 0/15.0k, 0/14.1k             │  pend fav : 0         │
│  known ints : 0/1005, 0/4150, 0/5794               │ own finds : 413       │
│  dictionary : 0/0, 0/0, 0/3974, 0/4083             │  imported : 254       │
│havoc/splice : 340/23.3M, 49/41.3M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 21/314k, 0/0         ├───────────────────────┘
│    trim/eff : 41.18%/74.2k, 97.35%                 │          [cpu002:  9%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++
```

Worker 4:

```bash
     AFL ++4.21c {fuzzer04} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 8 min, 9 sec       │  cycles done : 44    │
│   last new find : 0 days, 0 hrs, 8 min, 43 sec      │ corpus count : 1568  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1368.370 (87.2%)   │    map density : 0.10% / 0.42%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.38 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 8               │ favored items : 132 (8.42%)         │
│ stage execs : 24/37 (64.86%)         │  new edges on : 222 (14.16%)        │
│ total execs : 71.2M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 20.1k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/208, 0/207, 0/205                  │    levels : 12        │
│  byte flips : 0/26, 0/25, 0/23                     │   pending : 0         │
│ arithmetics : 0/1806, 0/3360, 0/3080               │  pend fav : 0         │
│  known ints : 0/232, 0/940, 0/1278                 │ own finds : 362       │
│  dictionary : 0/0, 0/0, 0/832, 0/864               │  imported : 297       │
│havoc/splice : 293/25.4M, 50/45.6M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 18/160k, 0/0         ├───────────────────────┘
│    trim/eff : 49.41%/71.3k, 96.15%                 │          [cpu003:  4%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++
```


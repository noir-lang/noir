
```bash
     AFL ++4.21c {fuzzer01} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 49 min, 46 sec     │  cycles done : 332   │
│   last new find : 0 days, 0 hrs, 3 min, 50 sec      │ corpus count : 7465  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7267*11 (97.3%)    │    map density : 0.10% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 142 (1.90%)         │
│ stage execs : 53/300 (17.67%)        │  new edges on : 251 (3.36%)         │
│ total execs : 70.0M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 23.9k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/600, 0/599, 0/597                  │    levels : 15        │
│  byte flips : 0/75, 0/74, 0/72                     │   pending : 0         │
│ arithmetics : 0/5235, 0/10.2k, 0/9940              │  pend fav : 0         │
│  known ints : 0/672, 0/2802, 0/4022                │ own finds : 554       │
│  dictionary : 0/0, 0/0, 0/750, 0/760               │  imported : 170       │
│havoc/splice : 451/24.9M, 81/44.9M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 21/15.8k, 0/0        ├───────────────────────┘
│    trim/eff : disabled, 98.67%                     │          [cpu000: 28%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++
```

```bash
     AFL ++4.21c {fuzzer02} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 49 min, 28 sec     │  cycles done : 7     │
│   last new find : 0 days, 0 hrs, 3 min, 28 sec      │ corpus count : 7465  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7177.351 (96.1%)   │    map density : 0.13% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 13              │ favored items : 150 (2.01%)         │
│ stage execs : 59/112 (52.68%)        │  new edges on : 257 (3.44%)         │
│ total execs : 60.1M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 20.2k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 12/96, 3/95, 2/93                    │    levels : 10        │
│  byte flips : 0/12, 1/11, 0/9                      │   pending : 0         │
│ arithmetics : 9/782, 0/989, 0/700                  │  pend fav : 0         │
│  known ints : 1/88, 2/375, 0/464                   │ own finds : 440       │
│  dictionary : 0/0, 0/0, 0/48, 0/52                 │  imported : 284       │
│havoc/splice : 342/21.6M, 59/38.3M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 5/54.2k, 0/0         ├───────────────────────┘
│    trim/eff : 46.71%/62.1k, 66.67%                 │          [cpu001: 25%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++
```

```bash

     AFL ++4.21c {fuzzer03} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 49 min, 3 sec      │  cycles done : 7     │
│   last new find : 0 days, 0 hrs, 3 min, 54 sec      │ corpus count : 7470  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7440.187 (99.6%)   │    map density : 0.11% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 2               │ favored items : 159 (2.13%)         │
│ stage execs : 21/37 (56.76%)         │  new edges on : 255 (3.41%)         │
│ total execs : 60.5M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 21.4k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 6/408, 4/406, 4/402                  │    levels : 16        │
│  byte flips : 0/51, 0/49, 0/45                     │   pending : 0         │
│ arithmetics : 6/3500, 0/6191, 0/5740               │  pend fav : 0         │
│  known ints : 0/440, 2/1810, 0/2480                │ own finds : 385       │
│  dictionary : 0/0, 0/0, 0/1604, 0/1652             │  imported : 344       │
│havoc/splice : 293/21.6M, 61/38.8M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 4/46.3k, 0/0         ├───────────────────────┘
│    trim/eff : 45.45%/65.1k, 90.20%                 │          [cpu002: 23%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++
```

```bash
     AFL ++4.21c {fuzzer04} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 49 min, 1 sec      │  cycles done : 8     │                                                                                                                                          
│   last new find : 0 days, 0 hrs, 1 min, 13 sec      │ corpus count : 7464  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7208.406 (96.6%)   │    map density : 0.15% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 2               │ favored items : 154 (2.06%)         │
│ stage execs : 47/168 (27.98%)        │  new edges on : 251 (3.36%)         │
│ total execs : 62.1M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 20.6k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/440, 0/439, 0/437                  │    levels : 18        │
│  byte flips : 0/55, 0/54, 0/52                     │   pending : 0         │
│ arithmetics : 0/3808, 0/7140, 0/6860               │  pend fav : 0         │
│  known ints : 0/487, 0/2022, 0/2882                │ own finds : 332       │
│  dictionary : 0/0, 0/0, 0/1320, 0/1344             │  imported : 391       │
│havoc/splice : 271/22.2M, 45/39.7M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 13/56.2k, 0/0        ├───────────────────────┘
│    trim/eff : 47.80%/65.9k, 94.55%                 │          [cpu003: 21%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++
```

```bash
     AFL ++4.21c {fuzzer05} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 46 min, 55 sec     │  cycles done : 8     │
│   last new find : 0 days, 0 hrs, 4 min, 57 sec      │ corpus count : 7456  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 6819.380 (91.5%)   │    map density : 0.07% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 148 (1.98%)         │
│ stage execs : 9/300 (3.00%)          │  new edges on : 254 (3.41%)         │
│ total execs : 58.9M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 21.5k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 3         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 149       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 566       │
│havoc/splice : 126/21.0M, 8/37.7M                   │ stability : 100.00%   │
│py/custom/rq : unused, unused, 15/53.4k, 0/0        ├───────────────────────┘
│    trim/eff : 45.51%/65.7k, n/a                    │          [cpu004: 64%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++
```

```bash
     AFL ++4.21c {fuzzer06} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 46 min, 29 sec     │  cycles done : 8     │
│   last new find : 0 days, 0 hrs, 1 min, 52 sec      │ corpus count : 7476  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7233.355 (96.7%)   │    map density : 0.08% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 8               │ favored items : 157 (2.10%)         │
│ stage execs : 8/37 (21.62%)          │  new edges on : 260 (3.48%)         │
│ total execs : 55.6M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 19.4k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 7         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 209       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 526       │
│havoc/splice : 177/20.0M, 29/35.4M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 3/57.3k, 0/0         ├───────────────────────┘
│    trim/eff : 45.84%/64.4k, n/a                    │          [cpu005: 17%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++
```

```bash
     AFL ++4.21c {fuzzer07} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 32 min, 2 sec      │  cycles done : 6     │
│   last new find : 0 days, 0 hrs, 0 min, 16 sec      │ corpus count : 7467  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 6938.218 (92.9%)   │    map density : 0.10% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 9               │ favored items : 150 (2.01%)         │
│ stage execs : 17/37 (45.95%)         │  new edges on : 249 (3.33%)         │
│ total execs : 40.5M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 21.5k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 3         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 141       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 585       │
│havoc/splice : 129/14.4M, 1/26.0M                   │ stability : 100.00%   │
│py/custom/rq : unused, unused, 11/42.5k, 0/0        ├───────────────────────┘
│    trim/eff : 45.47%/65.9k, n/a                    │          [cpu006: 51%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++
```

```bash
     AFL ++4.21c {fuzzer08} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 31 min, 51 sec     │  cycles done : 6     │
│   last new find : 0 days, 0 hrs, 3 min, 40 sec      │ corpus count : 7465  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 6772*15 (90.7%)    │    map density : 0.08% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 10              │ favored items : 152 (2.04%)         │
│ stage execs : 5/37 (13.51%)          │  new edges on : 252 (3.38%)         │
│ total execs : 40.4M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 21.8k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 2         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 137       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 587       │
│havoc/splice : 125/14.4M, 1/25.8M                   │ stability : 100.00%   │
│py/custom/rq : unused, unused, 11/50.0k, 0/0        ├───────────────────────┘
│    trim/eff : 45.33%/65.9k, n/a                    │          [cpu007: 10%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++
```

```bash
     AFL ++4.21c {fuzzer09} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 29 min, 34 sec     │  cycles done : 5     │
│   last new find : 0 days, 0 hrs, 2 min, 56 sec      │ corpus count : 7465  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 6965.196 (93.3%)   │    map density : 0.11% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 3               │ favored items : 146 (1.96%)         │
│ stage execs : 22/37 (59.46%)         │  new edges on : 256 (3.43%)         │
│ total execs : 36.7M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 21.3k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 3         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 138       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 586       │
│havoc/splice : 124/13.2M, 2/23.3M                   │ stability : 100.00%   │
│py/custom/rq : unused, unused, 12/50.9k, 0/0        ├───────────────────────┘
│    trim/eff : 45.31%/66.0k, n/a                    │          [cpu008:  7%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++
```

```bash
     AFL ++4.21c {fuzzer10} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 29 min, 25 sec     │  cycles done : 5     │
│   last new find : 0 days, 0 hrs, 1 min, 51 sec      │ corpus count : 7472  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7069*15 (94.6%)    │    map density : 0.13% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.39 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 155 (2.07%)         │
│ stage execs : 217/450 (48.22%)       │  new edges on : 258 (3.45%)         │
│ total execs : 35.5M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 21.4k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 3         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 157       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 574       │
│havoc/splice : 143/12.7M, 2/22.7M                   │ stability : 100.00%   │
│py/custom/rq : unused, unused, 12/52.3k, 0/0        ├───────────────────────┘
│    trim/eff : 45.42%/65.3k, n/a                    │          [cpu009:  3%]
│    trim/eff : 45.42%/65.3k, n/a                    │          [cpu009: 10%]
└─ strategy: explore ────────── state: in progress ──┘

+++ Testing aborted by user +++
```


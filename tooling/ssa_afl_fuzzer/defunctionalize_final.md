     AFL ++4.21c {fuzzer01} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 15 min, 20 sec     │  cycles done : 662   │
│   last new find : 0 days, 0 hrs, 9 min, 47 sec      │ corpus count : 1459  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 759*26 (52.0%)     │    map density : 0.11% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 15              │ favored items : 136 (9.32%)         │
│ stage execs : 11/37 (29.73%)         │  new edges on : 248 (17.00%)        │
│ total execs : 105M                   │ total crashes : 0 (0 saved)         │
│  exec speed : 23.9k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/784, 0/783, 0/781                  │    levels : 9         │
│  byte flips : 0/98, 0/97, 0/95                     │   pending : 0         │
│ arithmetics : 0/6846, 0/13.4k, 0/13.2k             │  pend fav : 0         │
│  known ints : 0/880, 0/3676, 0/5310                │ own finds : 497       │
│  dictionary : 0/0, 0/0, 0/1960, 0/1980             │  imported : 228       │
│havoc/splice : 382/37.3M, 85/67.6M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 29/172k, 0/0         ├───────────────────────┘
│    trim/eff : disabled, 98.98%                     │          [cpu000:  4%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer02} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 15 min, 16 sec     │  cycles done : 67    │
│   last new find : 0 days, 0 hrs, 9 min, 21 sec      │ corpus count : 1453  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1400*24 (96.4%)    │    map density : 0.08% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 3               │ favored items : 142 (9.77%)         │
│ stage execs : 33/50 (66.00%)         │  new edges on : 249 (17.14%)        │
│ total execs : 93.5M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 24.5k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 12/992, 3/990, 1/986                 │    levels : 12        │
│  byte flips : 0/124, 2/122, 0/118                  │   pending : 0         │
│ arithmetics : 10/8549, 0/15.8k, 0/15.3k            │  pend fav : 0         │
│  known ints : 2/1082, 3/4541, 0/6518               │ own finds : 490       │
│  dictionary : 0/0, 0/0, 0/2432, 0/2462             │  imported : 229       │
│havoc/splice : 347/33.6M, 80/59.6M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 21/255k, 0/0         ├───────────────────────┘
│    trim/eff : 32.63%/63.7k, 92.74%                 │          [cpu001:  7%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer03} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 14 min, 54 sec     │  cycles done : 85    │
│   last new find : 0 days, 0 hrs, 9 min, 32 sec      │ corpus count : 1446  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 886.704 (61.3%)    │    map density : 0.09% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 7               │ favored items : 140 (9.68%)         │
│ stage execs : 13/37 (35.14%)         │  new edges on : 253 (17.50%)        │
│ total execs : 100M                   │ total crashes : 0 (0 saved)         │
│  exec speed : 24.5k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 10        │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 342       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 370       │
│havoc/splice : 277/36.2M, 51/64.4M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 14/247k, 0/0         ├───────────────────────┘
│    trim/eff : 29.56%/72.5k, n/a                    │          [cpu004: 14%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer04} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 14 min, 31 sec     │  cycles done : 93    │
│   last new find : 0 days, 0 hrs, 9 min, 49 sec      │ corpus count : 1463  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1292.749 (88.3%)   │    map density : 0.11% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 146 (9.98%)         │
│ stage execs : 145/200 (72.50%)       │  new edges on : 253 (17.29%)        │
│ total execs : 98.7M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 24.1k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 4         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 198       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 531       │
│havoc/splice : 169/35.3M, 18/63.0M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 11/254k, 0/0         ├───────────────────────┘
│    trim/eff : 33.47%/59.5k, n/a                    │          [cpu005: 12%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer05} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 14 min, 1 sec      │  cycles done : 95    │
│   last new find : 0 days, 0 hrs, 9 min, 12 sec      │ corpus count : 1475  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1168*45 (79.2%)    │    map density : 0.07% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 147 (9.97%)         │
│ stage execs : 216/300 (72.00%)       │  new edges on : 256 (17.36%)        │
│ total execs : 102M                   │ total crashes : 0 (0 saved)         │
│  exec speed : 23.5k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 15/256, 4/255, 1/253                 │    levels : 4         │
│  byte flips : 0/32, 3/31, 0/29                     │   pending : 0         │
│ arithmetics : 12/2123, 0/3254, 0/2940              │  pend fav : 0         │
│  known ints : 2/257, 2/1095, 0/1544                │ own finds : 227       │
│  dictionary : 0/0, 0/0, 0/126, 0/132               │  imported : 514       │
│havoc/splice : 155/37.2M, 14/65.4M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 11/127k, 0/0         ├───────────────────────┘
│    trim/eff : 32.38%/61.9k, 75.00%                 │          [cpu006: 12%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer06} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 13 min, 39 sec     │  cycles done : 94    │
│   last new find : 0 days, 0 hrs, 9 min, 39 sec      │ corpus count : 1458  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1097.1036 (75.2%)  │    map density : 0.18% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 5               │ favored items : 143 (9.81%)         │
│ stage execs : 22/37 (59.46%)         │  new edges on : 250 (17.15%)        │
│ total execs : 102M                   │ total crashes : 0 (0 saved)         │
│  exec speed : 24.2k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 4         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 179       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 545       │
│havoc/splice : 144/36.5M, 23/65.3M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 12/168k, 0/0         ├───────────────────────┘
│    trim/eff : 29.26%/60.9k, n/a                    │          [cpu007: 20%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer07} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 13 min, 24 sec     │  cycles done : 81    │
│   last new find : 0 days, 0 hrs, 9 min, 2 sec       │ corpus count : 1467  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1049.739 (71.5%)   │    map density : 0.13% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 6               │ favored items : 145 (9.88%)         │
│ stage execs : 31/56 (55.36%)         │  new edges on : 256 (17.45%)        │
│ total execs : 99.9M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 23.4k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 4         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 173       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 560       │
│havoc/splice : 154/35.4M, 6/64.3M                   │ stability : 100.00%   │
│py/custom/rq : unused, unused, 13/175k, 0/0         ├───────────────────────┘
│    trim/eff : 27.14%/58.7k, n/a                    │          [cpu008: 25%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer08} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 13 min, 15 sec     │  cycles done : 83    │
│   last new find : 0 days, 0 hrs, 9 min, 35 sec      │ corpus count : 1457  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1449.523 (99.5%)   │    map density : 0.11% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 12              │ favored items : 146 (10.02%)        │
│ stage execs : 22/37 (59.46%)         │  new edges on : 254 (17.43%)        │
│ total execs : 99.7M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 23.4k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 3         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 158       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 565       │
│havoc/splice : 132/35.4M, 15/64.1M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 11/159k, 0/0         ├───────────────────────┘
│    trim/eff : 45.52%/59.1k, n/a                    │          [cpu009: 23%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++


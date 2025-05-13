
     AFL ++4.21c {fuzzer01} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 49 sec      │  cycles done : 480   │
│   last new find : 0 days, 0 hrs, 7 min, 22 sec      │ corpus count : 7502  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 6764*26 (90.2%)    │    map density : 0.07% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 141 (1.88%)         │
│ stage execs : 839/1398 (60.01%)      │  new edges on : 266 (3.55%)         │
│ total execs : 78.4M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 1793/sec               │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 3/5592, 3/5591, 2/5589               │    levels : 9         │
│  byte flips : 0/699, 1/698, 0/696                  │   pending : 0         │
│ arithmetics : 5/48.9k, 0/97.6k, 0/97.3k            │  pend fav : 0         │
│  known ints : 0/6285, 1/26.5k, 0/39.0k             │ own finds : 192       │
│  dictionary : 0/0, 0/0, 0/2796, 0/2800             │  imported : 569       │
│havoc/splice : 150/28.0M, 18/50.1M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 7/244k, 0/0          ├───────────────────────┘
│    trim/eff : disabled, 99.86%                     │          [cpu000: 28%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++


     AFL ++4.21c {fuzzer02} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 44 sec      │  cycles done : 6     │                                                                                                                                          
│   last new find : 0 days, 0 hrs, 9 min, 20 sec      │ corpus count : 7521  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 6940*21 (92.3%)    │    map density : 0.07% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.61 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 3               │ favored items : 149 (1.98%)         │
│ stage execs : 2/75 (2.67%)           │  new edges on : 264 (3.51%)         │
│ total execs : 80.7M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 19.6k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 8/448, 6/446, 2/442                  │    levels : 14        │
│  byte flips : 0/56, 0/54, 0/50                     │   pending : 0         │
│ arithmetics : 10/3835, 0/6754, 0/6160              │  pend fav : 0         │
│  known ints : 1/482, 2/1989, 0/2740                │ own finds : 648       │
│  dictionary : 0/0, 0/0, 0/660, 0/682               │  imported : 132       │
│havoc/splice : 498/28.7M, 106/51.8M                 │ stability : 100.00%   │
│py/custom/rq : unused, unused, 9/66.9k, 0/0         ├───────────────────────┘
│    trim/eff : 44.42%/69.2k, 89.29%                 │          [cpu001: 26%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer03} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 40 sec      │  cycles done : 7     │
│   last new find : 0 days, 0 hrs, 1 min, 16 sec      │ corpus count : 7491  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7033.415 (93.9%)   │    map density : 0.15% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 5               │ favored items : 148 (1.98%)         │
│ stage execs : 43/112 (38.39%)        │  new edges on : 261 (3.48%)         │
│ total execs : 79.8M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 19.6k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 2/280, 1/278, 0/274                  │    levels : 13        │
│  byte flips : 0/35, 0/33, 0/29                     │   pending : 0         │
│ arithmetics : 0/2408, 0/4243, 0/3640               │  pend fav : 0         │
│  known ints : 0/306, 0/1224, 0/1594                │ own finds : 623       │
│  dictionary : 0/0, 0/0, 0/420, 0/442               │  imported : 127       │
│havoc/splice : 472/28.4M, 131/51.2M                 │ stability : 100.00%   │
│py/custom/rq : unused, unused, 14/59.0k, 0/0        ├───────────────────────┘
│    trim/eff : 45.44%/67.4k, 91.43%                 │          [cpu004: 25%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer04} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 36 sec      │  cycles done : 6     │
│   last new find : 0 days, 0 hrs, 6 min, 4 sec       │ corpus count : 7474  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7277.252 (97.4%)   │    map density : 0.10% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 14              │ favored items : 146 (1.95%)         │
│ stage execs : 27/75 (36.00%)         │  new edges on : 261 (3.49%)         │
│ total execs : 78.4M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 19.1k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/968, 0/965, 0/959                  │    levels : 15        │
│  byte flips : 0/121, 0/118, 0/112                  │   pending : 0         │
│ arithmetics : 0/8428, 0/16.1k, 0/15.5k             │  pend fav : 0         │
│  known ints : 0/1076, 1/4454, 0/6262               │ own finds : 601       │
│  dictionary : 0/0, 0/0, 0/2790, 0/2936             │  imported : 132       │
│havoc/splice : 461/28.2M, 112/50.0M                 │ stability : 100.00%   │
│py/custom/rq : unused, unused, 24/63.7k, 0/0        ├───────────────────────┘
│    trim/eff : 45.95%/67.8k, 97.52%                 │          [cpu005: 21%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer05} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 31 sec      │  cycles done : 6     │
│   last new find : 0 days, 0 hrs, 0 min, 18 sec      │ corpus count : 7489  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 6916.212 (92.3%)   │    map density : 0.08% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 13              │ favored items : 148 (1.98%)         │
│ stage execs : 41/75 (54.67%)         │  new edges on : 253 (3.38%)         │
│ total execs : 82.8M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 20.5k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 15        │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 0         │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 0         │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 627       │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 121       │
│havoc/splice : 515/30.1M, 93/52.5M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 19/66.6k, 0/0        ├───────────────────────┘
│    trim/eff : 45.19%/69.0k, n/a                    │          [cpu006: 15%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer06} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 27 sec      │  cycles done : 7     │
│   last new find : 0 days, 0 hrs, 0 min, 49 sec      │ corpus count : 7491  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7200.365 (96.1%)   │    map density : 0.15% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 6               │ favored items : 146 (1.95%)         │
│ stage execs : 32/56 (57.14%)         │  new edges on : 259 (3.46%)         │
│ total execs : 78.8M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 20.8k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/1152, 0/1149, 0/1143               │    levels : 13        │
│  byte flips : 0/144, 0/141, 0/135                  │   pending : 0         │
│ arithmetics : 0/10.0k, 0/19.2k, 0/18.6k            │  pend fav : 0         │
│  known ints : 0/1283, 1/5317, 0/7540               │ own finds : 613       │
│  dictionary : 0/0, 0/0, 1/5752, 0/6006             │  imported : 137       │
│havoc/splice : 482/27.9M, 105/50.7M                 │ stability : 100.00%   │
│py/custom/rq : unused, unused, 20/69.1k, 0/0        ├───────────────────────┘
│    trim/eff : 45.27%/67.3k, 97.22%                 │          [cpu007: 17%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer07} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 21 sec      │  cycles done : 6     │
│   last new find : 0 days, 0 hrs, 7 min, 0 sec       │ corpus count : 7490  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7177.390 (95.8%)   │    map density : 0.12% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 151 (2.02%)         │
│ stage execs : 1159/1350 (85.85%)     │  new edges on : 263 (3.51%)         │
│ total execs : 78.5M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 19.1k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/1232, 0/1229, 0/1223               │    levels : 16        │
│  byte flips : 0/154, 0/151, 0/145                  │   pending : 0         │
│ arithmetics : 0/10.6k, 0/19.7k, 0/18.9k            │  pend fav : 0         │
│  known ints : 0/1355, 0/5638, 0/8020               │ own finds : 628       │
│  dictionary : 0/0, 0/0, 1/8444, 0/8602             │  imported : 121       │
│havoc/splice : 505/27.9M, 100/50.4M                 │ stability : 100.00%   │
│py/custom/rq : unused, unused, 12/72.7k, 0/0        ├───────────────────────┘
│    trim/eff : 45.19%/65.9k, 93.51%                 │          [cpu008: 21%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer08} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 17 sec      │  cycles done : 8     │
│   last new find : 0 days, 0 hrs, 5 min, 44 sec      │ corpus count : 7498  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7113.386 (94.9%)   │    map density : 0.12% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 11              │ favored items : 154 (2.05%)         │
│ stage execs : 22/112 (19.64%)        │  new edges on : 260 (3.47%)         │
│ total execs : 78.2M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 20.0k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 1/224, 0/223, 0/221                  │    levels : 13        │
│  byte flips : 0/28, 0/27, 0/25                     │   pending : 0         │
│ arithmetics : 0/1904, 0/3220, 0/2940               │  pend fav : 0         │
│  known ints : 0/236, 2/986, 0/1360                 │ own finds : 444       │
│  dictionary : 0/0, 0/0, 3/1652, 0/1711             │  imported : 313       │
│havoc/splice : 341/28.0M, 85/50.1M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 8/62.5k, 0/0         ├───────────────────────┘
│    trim/eff : 44.42%/66.8k, 85.71%                 │          [cpu009: 10%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer09} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 13 sec      │  cycles done : 8     │
│   last new find : 0 days, 0 hrs, 4 min, 59 sec      │ corpus count : 7488  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 7305.345 (97.6%)   │    map density : 0.11% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 15              │ favored items : 148 (1.98%)         │
│ stage execs : 89/112 (79.46%)        │  new edges on : 259 (3.46%)         │
│ total execs : 80.4M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 20.6k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/2440, 0/2438, 0/2434               │    levels : 12        │
│  byte flips : 0/305, 0/303, 0/299                  │   pending : 0         │
│ arithmetics : 0/21.2k, 0/41.0k, 0/40.6k            │  pend fav : 0         │
│  known ints : 0/2717, 1/11.4k, 0/16.7k             │ own finds : 423       │
│  dictionary : 0/0, 0/0, 0/20.6k, 0/20.9k           │  imported : 324       │
│havoc/splice : 321/28.9M, 87/51.4M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 4/65.7k, 0/0         ├───────────────────────┘
│    trim/eff : 45.55%/66.6k, 96.72%                 │          [cpu010:  7%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer10} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 7 min, 8 sec       │  cycles done : 9     │
│   last new find : 0 days, 0 hrs, 6 min, 16 sec      │ corpus count : 7476  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 6991.560 (93.5%)   │    map density : 0.10% / 0.43%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.62 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 5               │ favored items : 150 (2.01%)         │
│ stage execs : 11/37 (29.73%)         │  new edges on : 258 (3.45%)         │
│ total execs : 78.5M                  │ total crashes : 0 (0 saved)         │
│  exec speed : 20.7k/sec              │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 7/192, 3/190, 4/186                  │    levels : 11        │
│  byte flips : 0/24, 1/22, 0/18                     │   pending : 0         │
│ arithmetics : 9/1565, 0/1969, 0/1540               │  pend fav : 0         │
│  known ints : 1/182, 3/752, 0/938                  │ own finds : 385       │
│  dictionary : 0/0, 0/0, 0/460, 0/545               │  imported : 350       │
│havoc/splice : 287/28.3M, 56/50.0M                  │ stability : 100.00%   │
│py/custom/rq : unused, unused, 6/59.7k, 0/0         ├───────────────────────┘
│    trim/eff : 46.12%/64.7k, 66.67%                 │          [cpu011:  4%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++


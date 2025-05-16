
     AFL ++4.21c {fuzzer01} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 56 sec     │  cycles done : 1953  │
│   last new find : 0 days, 0 hrs, 8 min, 34 sec      │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 6 min, 15 sec      │saved crashes : 400   │
│ last saved hang : 0 days, 0 hrs, 46 min, 15 sec     │  saved hangs : 166   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 10.9k*71 (72.9%)   │    map density : 0.74% / 3.97%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.34 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 848 (5.68%)         │
│ stage execs : 21.5k/28.8k (74.56%)   │  new edges on : 1398 (9.37%)        │
│ total execs : 868M                   │ total crashes : 7888 (400 saved)    │
│  exec speed : 84.62/sec (slow!)      │  total tmouts : 93.9k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 2/81.8k, 2/81.8k, 1/81.8k            │    levels : 10        │
│  byte flips : 0/10.2k, 0/10.2k, 0/10.2k            │   pending : 0         │
│ arithmetics : 1/715k, 0/1.42M, 0/1.42M             │  pend fav : 0         │
│  known ints : 0/91.9k, 0/387k, 0/570k              │ own finds : 1393      │
│  dictionary : 0/0, 0/0, 1/6.57M, 0/6.58M           │  imported : 5148      │
│havoc/splice : 826/292M, 188/563M                   │ stability : 98.27%    │
│py/custom/rq : unused, unused, 729/11.7M, 0/0       ├───────────────────────┘
│    trim/eff : disabled, 99.59%                     │          [cpu000: 28%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer02} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 51 sec     │  cycles done : 144   │
│   last new find : 0 days, 0 hrs, 9 min, 12 sec      │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 5 min, 50 sec      │saved crashes : 412   │
│ last saved hang : 0 days, 6 hrs, 8 min, 37 sec      │  saved hangs : 163   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 12.7k*85 (85.1%)   │    map density : 1.01% / 3.96%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.34 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 867 (5.82%)         │
│ stage execs : 29.5k/182k (16.17%)    │  new edges on : 1406 (9.45%)        │
│ total execs : 758M                   │ total crashes : 11.0k (412 saved)   │
│  exec speed : 137.2/sec              │  total tmouts : 32.8k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 4/739k, 0/739k, 2/739k               │    levels : 14        │
│  byte flips : 0/92.5k, 3/92.4k, 0/92.3k            │   pending : 0         │
│ arithmetics : 9/6.47M, 0/12.9M, 0/12.9M            │  pend fav : 0         │
│  known ints : 0/831k, 0/3.51M, 0/5.17M             │ own finds : 2293      │
│  dictionary : 0/0, 0/0, 4/188M, 0/188M             │  imported : 4208      │
│havoc/splice : 1408/248M, 333/480M                  │ stability : 98.28%    │
│py/custom/rq : unused, unused, 835/23.4M, 0/0       ├───────────────────────┘
│    trim/eff : 1.87%/4.03M, 99.89%                  │          [cpu001: 34%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer03} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 51 sec     │  cycles done : 135   │
│   last new find : 0 days, 0 hrs, 2 min, 53 sec      │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 1 min, 16 sec      │saved crashes : 386   │
│ last saved hang : 0 days, 0 hrs, 30 min, 26 sec     │  saved hangs : 169   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 14.9k.37 (99.9%)   │    map density : 0.96% / 3.97%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.33 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 11              │ favored items : 847 (5.69%)         │
│ stage execs : 48/50 (96.00%)         │  new edges on : 1406 (9.44%)        │
│ total execs : 708M                   │ total crashes : 8657 (386 saved)    │
│  exec speed : 18.1k/sec              │  total tmouts : 55.7k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 3/658k, 2/658k, 1/658k               │    levels : 14        │
│  byte flips : 0/82.3k, 2/82.3k, 0/82.3k            │   pending : 0         │
│ arithmetics : 5/5.76M, 0/11.5M, 0/11.5M            │  pend fav : 0         │
│  known ints : 0/740k, 0/3.13M, 0/4.61M             │ own finds : 2232      │
│  dictionary : 0/0, 0/0, 5/174M, 0/174M             │  imported : 4281      │
│havoc/splice : 1337/231M, 290/448M                  │ stability : 98.30%    │
│py/custom/rq : unused, unused, 875/22.5M, 0/0       ├───────────────────────┘
│    trim/eff : 1.83%/4.06M, 99.89%                  │          [cpu005: 25%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer04} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 51 sec     │  cycles done : 127   │
│   last new find : 0 days, 0 hrs, 3 min, 5 sec       │ corpus count : 14.9k │
│last saved crash : 0 days, 1 hrs, 41 min, 15 sec     │saved crashes : 377   │
│ last saved hang : 0 days, 5 hrs, 7 min, 12 sec      │  saved hangs : 171   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 8805*106 (59.1%)   │    map density : 0.57% / 3.97%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.33 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 15              │ favored items : 852 (5.72%)         │
│ stage execs : 31/50 (62.00%)         │  new edges on : 1399 (9.39%)        │
│ total execs : 672M                   │ total crashes : 8712 (377 saved)    │
│  exec speed : 18.3k/sec              │  total tmouts : 17.8k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 5/474k, 2/474k, 1/474k               │    levels : 16        │
│  byte flips : 0/59.3k, 2/59.3k, 0/59.2k            │   pending : 0         │
│ arithmetics : 7/4.15M, 0/8.29M, 0/8.28M            │  pend fav : 0         │
│  known ints : 0/533k, 0/2.25M, 0/3.32M             │ own finds : 2128      │
│  dictionary : 0/0, 0/0, 5/100M, 0/100M             │  imported : 4386      │
│havoc/splice : 1259/220M, 352/425M                  │ stability : 98.23%    │
│py/custom/rq : unused, unused, 780/21.8M, 0/0       ├───────────────────────┘
│    trim/eff : 1.79%/4.05M, 99.85%                  │          [cpu006: 23%]
└─ strategy: explore ────────── state: in progress ──┘

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer05} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 51 sec     │  cycles done : 135   │
│   last new find : 0 days, 0 hrs, 20 min, 56 sec     │ corpus count : 15.0k │
│last saved crash : 0 days, 0 hrs, 21 min, 30 sec     │saved crashes : 385   │
│ last saved hang : 0 days, 8 hrs, 47 min, 6 sec      │  saved hangs : 152   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 12.8k*65 (85.6%)   │    map density : 0.80% / 3.96%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.34 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 863 (5.77%)         │
│ stage execs : 30.1k/65.7k (45.79%)   │  new edges on : 1399 (9.35%)        │
│ total execs : 734M                   │ total crashes : 9225 (385 saved)    │
│  exec speed : 458.7/sec              │  total tmouts : 7837 (0 saved)      │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 13/853k, 4/853k, 2/853k              │    levels : 17        │
│  byte flips : 0/106k, 2/106k, 0/106k               │   pending : 0         │
│ arithmetics : 8/7.46M, 0/14.9M, 0/14.9M            │  pend fav : 0         │
│  known ints : 0/959k, 1/4.05M, 0/5.96M             │ own finds : 2239      │
│  dictionary : 0/0, 0/0, 12/265M, 0/265M            │  imported : 4344      │
│havoc/splice : 1350/241M, 273/464M                  │ stability : 98.28%    │
│py/custom/rq : unused, unused, 824/23.4M, 0/0       ├───────────────────────┘
│    trim/eff : 1.98%/4.12M, 99.88%                  │          [cpu007: 21%]
└─ strategy: exploit ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer06} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 50 sec     │  cycles done : 122   │
│   last new find : 0 days, 1 hrs, 17 min, 51 sec     │ corpus count : 14.9k │
│last saved crash : 0 days, 1 hrs, 1 min, 39 sec      │saved crashes : 377   │
│ last saved hang : 0 days, 7 hrs, 37 min, 35 sec     │  saved hangs : 166   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 14.8k*6 (99.7%)    │    map density : 0.76% / 3.97%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.32 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 849 (5.71%)         │
│ stage execs : 248k/323k (77.00%)     │  new edges on : 1392 (9.35%)        │
│ total execs : 655M                   │ total crashes : 8869 (377 saved)    │
│  exec speed : 64.88/sec (slow!)      │  total tmouts : 84.7k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 9/829k, 9/829k, 0/829k               │    levels : 23        │
│  byte flips : 0/103k, 1/103k, 0/103k               │   pending : 0         │
│ arithmetics : 10/7.26M, 0/14.5M, 0/14.5M           │  pend fav : 0         │
│  known ints : 0/932k, 0/3.94M, 0/5.80M             │ own finds : 2122      │
│  dictionary : 0/0, 0/0, 8/234M, 0/234M             │  imported : 4374      │
│havoc/splice : 1183/215M, 344/414M                  │ stability : 98.20%    │
│py/custom/rq : unused, unused, 819/20.6M, 0/0       ├───────────────────────┘
│    trim/eff : 1.84%/4.05M, 99.89%                  │          [cpu008: 21%]
└─ strategy: exploit ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++


     AFL ++4.21c {fuzzer07} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 50 sec     │  cycles done : 114   │
│   last new find : 0 days, 0 hrs, 3 min, 37 sec      │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 0 min, 7 sec       │saved crashes : 385   │
│ last saved hang : 0 days, 1 hrs, 47 min, 14 sec     │  saved hangs : 167   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 11.8k.1575 (79.5%) │    map density : 0.67% / 3.96%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.34 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 4               │ favored items : 863 (5.79%)         │
│ stage execs : 11/12 (91.67%)         │  new edges on : 1393 (9.35%)        │
│ total execs : 586M                   │ total crashes : 8112 (385 saved)    │
│  exec speed : 17.8k/sec              │  total tmouts : 131k (0 saved)      │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 14/443k, 5/443k, 3/443k              │    levels : 14        │
│  byte flips : 0/55.4k, 1/55.4k, 0/55.3k            │   pending : 0         │
│ arithmetics : 6/3.88M, 0/7.74M, 0/7.73M            │  pend fav : 0         │
│  known ints : 0/498k, 0/2.10M, 0/3.10M             │ own finds : 2108      │
│  dictionary : 0/0, 0/0, 4/112M, 0/112M             │  imported : 4401      │
│havoc/splice : 1209/190M, 367/367M                  │ stability : 98.25%    │
│py/custom/rq : unused, unused, 762/22.7M, 0/0       ├───────────────────────┘
│    trim/eff : 1.94%/4.10M, 99.78%                  │          [cpu009: 18%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer08} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 50 sec     │  cycles done : 116   │
│   last new find : 0 days, 0 hrs, 2 min, 8 sec       │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 34 min, 35 sec     │saved crashes : 387   │
│ last saved hang : 0 days, 3 hrs, 3 min, 24 sec      │  saved hangs : 167   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 11.6k*54 (77.6%)   │    map density : 0.73% / 3.96%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.34 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 858 (5.75%)         │
│ stage execs : 10.1k/20.6k (49.10%)   │  new edges on : 1397 (9.37%)        │
│ total execs : 626M                   │ total crashes : 9664 (387 saved)    │
│  exec speed : 1028/sec               │  total tmouts : 29.7k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 10/477k, 4/477k, 0/477k              │    levels : 19        │
│  byte flips : 0/59.7k, 2/59.7k, 0/59.6k            │   pending : 0         │
│ arithmetics : 2/4.18M, 0/8.34M, 0/8.33M            │  pend fav : 0         │
│  known ints : 0/536k, 0/2.27M, 0/3.34M             │ own finds : 2290      │
│  dictionary : 0/0, 0/0, 9/115M, 0/115M             │  imported : 4239      │
│havoc/splice : 1275/205M, 435/395M                  │ stability : 98.27%    │
│py/custom/rq : unused, unused, 845/19.9M, 0/0       ├───────────────────────┘
│    trim/eff : 2.02%/4.08M, 99.85%                  │          [cpu010: 12%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer09} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 50 sec     │  cycles done : 135   │
│   last new find : 0 days, 0 hrs, 4 min, 36 sec      │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 26 min, 0 sec      │saved crashes : 405   │
│ last saved hang : 0 days, 0 hrs, 55 min, 20 sec     │  saved hangs : 160   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 14.9k*2 (100.0%)   │    map density : 1.43% / 3.97%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.33 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 864 (5.80%)         │
│ stage execs : 930/1642 (56.64%)      │  new edges on : 1389 (9.33%)        │
│ total execs : 726M                   │ total crashes : 10.8k (405 saved)   │
│  exec speed : 16.5k/sec              │  total tmouts : 31.8k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 4/520k, 1/520k, 2/520k               │    levels : 14        │
│  byte flips : 0/65.1k, 2/65.1k, 0/65.0k            │   pending : 0         │
│ arithmetics : 3/4.56M, 0/9.10M, 0/9.09M            │  pend fav : 0         │
│  known ints : 0/585k, 0/2.47M, 0/3.64M             │ own finds : 2049      │
│  dictionary : 0/0, 0/0, 7/119M, 0/119M             │  imported : 4455      │
│havoc/splice : 1125/238M, 418/459M                  │ stability : 98.27%    │
│py/custom/rq : unused, unused, 799/22.5M, 0/0       ├───────────────────────┘
│    trim/eff : 1.92%/4.04M, 99.86%                  │          [cpu012: 12%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer10} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 50 sec     │  cycles done : 131   │
│   last new find : 0 days, 0 hrs, 11 min, 35 sec     │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 35 min, 9 sec      │saved crashes : 386   │
│ last saved hang : 0 days, 3 hrs, 47 min, 21 sec     │  saved hangs : 192   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 12.3k*56 (82.1%)   │    map density : 0.73% / 3.97%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.33 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 869 (5.83%)         │
│ stage execs : 47.6k/182k (26.10%)    │  new edges on : 1393 (9.34%)        │
│ total execs : 696M                   │ total crashes : 9229 (386 saved)    │
│  exec speed : 124.9/sec              │  total tmouts : 19.8k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 6/495k, 5/495k, 1/495k               │    levels : 17        │
│  byte flips : 0/61.9k, 3/61.9k, 0/61.8k            │   pending : 0         │
│ arithmetics : 7/4.33M, 0/8.65M, 0/8.64M            │  pend fav : 0         │
│  known ints : 0/556k, 0/2.35M, 0/3.46M             │ own finds : 2268      │
│  dictionary : 0/0, 0/0, 2/115M, 0/115M             │  imported : 4261      │
│havoc/splice : 1348/227M, 320/439M                  │ stability : 98.25%    │
│py/custom/rq : unused, unused, 853/24.0M, 0/0       ├───────────────────────┘
│    trim/eff : 1.77%/4.10M, 99.83%                  │          [cpu013: 12%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer11} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 50 sec     │  cycles done : 136   │
│   last new find : 0 days, 0 hrs, 8 min, 46 sec      │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 5 min, 4 sec       │saved crashes : 396   │
│ last saved hang : 0 days, 6 hrs, 28 min, 47 sec     │  saved hangs : 163   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 11.0k*75 (73.5%)   │    map density : 1.22% / 3.97%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.33 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 868 (5.82%)         │
│ stage execs : 13.9k/42.7k (32.56%)   │  new edges on : 1398 (9.37%)        │
│ total execs : 720M                   │ total crashes : 9730 (396 saved)    │
│  exec speed : 63.83/sec (slow!)      │  total tmouts : 13.2k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 5/400k, 3/400k, 0/400k               │    levels : 24        │
│  byte flips : 0/50.0k, 2/50.0k, 0/49.9k            │   pending : 0         │
│ arithmetics : 8/3.50M, 0/6.99M, 0/6.98M            │  pend fav : 0         │
│  known ints : 0/449k, 0/1.90M, 0/2.80M             │ own finds : 2387      │
│  dictionary : 0/0, 0/0, 11/94.4M, 0/94.4M          │  imported : 4142      │
│havoc/splice : 1482/236M, 311/455M                  │ stability : 98.28%    │
│py/custom/rq : unused, unused, 854/23.5M, 0/0       ├───────────────────────┘
│    trim/eff : 1.93%/4.04M, 99.82%                  │          [cpu014:  9%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer12} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 1 days, 0 hrs, 15 min, 50 sec     │  cycles done : 127   │
│   last new find : 0 days, 0 hrs, 9 min, 52 sec      │ corpus count : 14.9k │
│last saved crash : 0 days, 0 hrs, 6 min, 1 sec       │saved crashes : 382   │
│ last saved hang : 0 days, 0 hrs, 33 min, 37 sec     │  saved hangs : 173   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 13.8k*45 (92.4%)   │    map density : 0.52% / 3.96%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 5.34 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 859 (5.76%)         │
│ stage execs : 3018/124k (2.43%)      │  new edges on : 1396 (9.37%)        │
│ total execs : 671M                   │ total crashes : 8545 (382 saved)    │
│  exec speed : 380.0/sec              │  total tmouts : 46.4k (0 saved)     │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 9/376k, 4/376k, 0/376k               │    levels : 14        │
│  byte flips : 0/47.1k, 2/47.1k, 0/47.0k            │   pending : 0         │
│ arithmetics : 6/3.30M, 0/6.58M, 0/6.57M            │  pend fav : 0         │
│  known ints : 0/423k, 0/1.79M, 0/2.63M             │ own finds : 2193      │
│  dictionary : 0/0, 0/0, 7/84.6M, 0/84.7M           │  imported : 4328      │
│havoc/splice : 1220/220M, 451/424M                  │ stability : 98.26%    │
│py/custom/rq : unused, unused, 790/20.4M, 0/0       ├───────────────────────┘
│    trim/eff : 2.01%/4.07M, 99.83%                  │          [cpu015:  3%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++



     AFL ++4.21c {fuzzer01} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 23 min, 6 sec      │  cycles done : 189   │
│   last new find : 0 days, 0 hrs, 4 min, 59 sec      │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 5 min, 10 sec      │saved crashes : 323   │
│ last saved hang : 0 days, 0 hrs, 8 min, 50 sec      │  saved hangs : 95    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 12.3k.0 (100.0%)   │    map density : 0.74% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 684 (5.56%)         │
│ stage execs : 53.8k/131k (40.97%)    │  new edges on : 1077 (8.76%)        │
│ total execs : 39.2M                  │ total crashes : 5901 (323 saved)    │
│  exec speed : 172.1/sec              │  total tmouts : 433 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/51.3k, 1/51.3k, 0/51.3k            │    levels : 11        │
│  byte flips : 0/6415, 0/6408, 0/6394               │   pending : 17        │
│ arithmetics : 2/448k, 0/893k, 0/891k               │  pend fav : 1         │
│  known ints : 0/57.6k, 0/243k, 0/357k              │ own finds : 1290      │
│  dictionary : 0/0, 0/0, 2/4.06M, 0/4.07M           │  imported : 2627      │
│havoc/splice : 512/12.7M, 48/23.9M                  │ stability : 98.29%    │
│py/custom/rq : unused, unused, 1003/2.27M, 0/0      ├───────────────────────┘
│    trim/eff : disabled, 99.58%                     │          [cpu000: 29%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer02} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 23 min, 6 sec      │  cycles done : 11    │
│   last new find : 0 days, 0 hrs, 0 min, 27 sec      │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 3 min, 38 sec      │saved crashes : 296   │
│ last saved hang : 0 days, 0 hrs, 0 min, 49 sec      │  saved hangs : 80    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1596*6 (13.0%)     │    map density : 1.13% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 15              │ favored items : 678 (5.51%)         │
│ stage execs : 2/12 (16.67%)          │  new edges on : 1062 (8.64%)        │
│ total execs : 54.0M                  │ total crashes : 3335 (296 saved)    │
│  exec speed : 17.7k/sec              │  total tmouts : 379 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 7/254k, 0/254k, 0/254k               │    levels : 19        │
│  byte flips : 0/31.8k, 4/31.8k, 0/31.8k            │   pending : 3         │
│ arithmetics : 6/2.23M, 0/4.44M, 0/4.43M            │  pend fav : 0         │
│  known ints : 0/285k, 1/1.21M, 0/1.78M             │ own finds : 2064      │
│  dictionary : 0/0, 0/0, 1/52.1M, 0/52.1M           │  imported : 1846      │
│havoc/splice : 1147/17.4M, 214/32.3M                │ stability : 98.28%    │
│py/custom/rq : unused, unused, 887/1.28M, 0/0       ├───────────────────────┘
│    trim/eff : 4.89%/2.66M, 99.75%                  │          [cpu001: 28%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer03} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 23 min, 6 sec      │  cycles done : 10    │
│   last new find : 0 days, 0 hrs, 0 min, 36 sec      │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 0 min, 53 sec      │saved crashes : 314   │
│ last saved hang : 0 days, 0 hrs, 0 min, 56 sec      │  saved hangs : 81    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 8779.127 (71.3%)   │    map density : 0.27% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 688 (5.59%)         │
│ stage execs : 65/75 (86.67%)         │  new edges on : 1065 (8.65%)        │
│ total execs : 49.0M                  │ total crashes : 3274 (314 saved)    │
│  exec speed : 16.6k/sec              │  total tmouts : 415 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 12/218k, 5/218k, 2/217k              │    levels : 17        │
│  byte flips : 0/27.3k, 2/27.2k, 0/27.2k            │   pending : 6         │
│ arithmetics : 9/1.91M, 0/3.80M, 0/3.79M            │  pend fav : 0         │
│  known ints : 0/244k, 0/1.03M, 0/1.52M             │ own finds : 2046      │
│  dictionary : 0/0, 0/0, 0/31.9M, 0/31.9M           │  imported : 1887      │
│havoc/splice : 1106/15.6M, 142/29.0M                │ stability : 98.29%    │
│py/custom/rq : unused, unused, 966/1.30M, 0/0       ├───────────────────────┘
│    trim/eff : 5.06%/2.61M, 99.67%                  │          [cpu005: 23%]
└─ strategy: explore ────────── state: in progress ──┘^C

     AFL ++4.21c {fuzzer04} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 23 min, 6 sec      │  cycles done : 11    │
│   last new find : 0 days, 0 hrs, 2 min, 19 sec      │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 0 min, 15 sec      │saved crashes : 324   │
│ last saved hang : 0 days, 0 hrs, 5 min, 47 sec      │  saved hangs : 86    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 9046.105 (73.7%)   │    map density : 0.08% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 6               │ favored items : 686 (5.59%)         │
│ stage execs : 9/12 (75.00%)          │  new edges on : 1071 (8.72%)        │
│ total execs : 52.7M                  │ total crashes : 3265 (324 saved)    │
│  exec speed : 18.1k/sec              │  total tmouts : 315 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 10/239k, 0/239k, 1/239k              │    levels : 18        │
│  byte flips : 0/30.0k, 2/29.9k, 0/29.9k            │   pending : 0         │
│ arithmetics : 13/2.10M, 0/4.17M, 0/4.17M           │  pend fav : 0         │
│  known ints : 0/269k, 0/1.14M, 0/1.67M             │ own finds : 2041      │
│  dictionary : 0/0, 0/0, 7/41.6M, 0/41.6M           │  imported : 1853      │
│havoc/splice : 1128/16.8M, 129/31.5M                │ stability : 98.27%    │
│py/custom/rq : unused, unused, 938/1.31M, 0/0       ├───────────────────────┘
│    trim/eff : 4.53%/2.64M, 99.59%                  │          [cpu006: 26%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer05} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 23 min, 5 sec      │  cycles done : 10    │
│   last new find : 0 days, 0 hrs, 1 min, 18 sec      │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 0 min, 28 sec      │saved crashes : 308   │
│ last saved hang : 0 days, 0 hrs, 1 min, 5 sec       │  saved hangs : 87    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 11.3k.123 (91.5%)  │    map density : 1.10% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 14              │ favored items : 690 (5.61%)         │
│ stage execs : 13/18 (72.22%)         │  new edges on : 1066 (8.67%)        │
│ total execs : 48.4M                  │ total crashes : 3207 (308 saved)    │
│  exec speed : 17.8k/sec              │  total tmouts : 447 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 12/108k, 7/108k, 1/108k              │    levels : 22        │
│  byte flips : 0/13.6k, 2/13.6k, 0/13.6k            │   pending : 4         │
│ arithmetics : 6/952k, 0/1.89M, 0/1.89M             │  pend fav : 0         │
│  known ints : 0/122k, 0/515k, 0/758k               │ own finds : 2013      │
│  dictionary : 0/0, 0/0, 2/8.93M, 0/8.94M           │  imported : 1897      │
│havoc/splice : 1047/15.3M, 149/28.8M                │ stability : 98.28%    │
│py/custom/rq : unused, unused, 978/1.30M, 0/0       ├───────────────────────┘
│    trim/eff : 4.11%/2.61M, 99.27%                  │          [cpu007: 25%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer06} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 23 min, 5 sec      │  cycles done : 10    │
│   last new find : 0 days, 0 hrs, 0 min, 10 sec      │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 0 min, 9 sec       │saved crashes : 323   │
│ last saved hang : 0 days, 0 hrs, 0 min, 15 sec      │  saved hangs : 100   │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 9544.148 (77.6%)   │    map density : 1.29% / 3.76%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.74 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 6               │ favored items : 695 (5.65%)         │
│ stage execs : 33/37 (89.19%)         │  new edges on : 1073 (8.72%)        │
│ total execs : 47.0M                  │ total crashes : 3479 (323 saved)    │
│  exec speed : 17.5k/sec              │  total tmouts : 738 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 6/146k, 6/146k, 1/146k               │    levels : 17        │
│  byte flips : 0/18.3k, 3/18.3k, 1/18.2k            │   pending : 6         │
│ arithmetics : 9/1.28M, 0/2.54M, 0/2.54M            │  pend fav : 0         │
│  known ints : 0/164k, 0/692k, 0/1.02M              │ own finds : 2073      │
│  dictionary : 0/0, 0/0, 7/18.2M, 0/18.3M           │  imported : 1849      │
│havoc/splice : 1077/14.9M, 128/27.7M                │ stability : 98.31%    │
│py/custom/rq : unused, unused, 1029/1.29M, 0/0      ├───────────────────────┘
│    trim/eff : 4.92%/2.63M, 99.39%                  │          [cpu008: 17%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer07} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 23 min, 5 sec      │  cycles done : 9     │
│   last new find : 0 days, 0 hrs, 0 min, 5 sec       │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 0 min, 0 sec       │saved crashes : 319   │
│ last saved hang : 0 days, 0 hrs, 0 min, 38 sec      │  saved hangs : 73    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 10.2k.131 (83.2%)  │    map density : 1.42% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 691 (5.62%)         │
│ stage execs : 75/150 (50.00%)        │  new edges on : 1060 (8.62%)        │
│ total execs : 45.7M                  │ total crashes : 3216 (319 saved)    │
│  exec speed : 17.7k/sec              │  total tmouts : 656 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 5/142k, 7/142k, 0/142k               │    levels : 19        │
│  byte flips : 0/17.8k, 1/17.7k, 0/17.7k            │   pending : 2         │
│ arithmetics : 9/1.24M, 0/2.47M, 0/2.47M            │  pend fav : 0         │
│  known ints : 0/159k, 0/673k, 0/989k               │ own finds : 2099      │
│  dictionary : 0/0, 0/0, 8/18.1M, 0/18.1M           │  imported : 1820      │
│havoc/splice : 1012/14.5M, 282/26.9M                │ stability : 98.27%    │
│py/custom/rq : unused, unused, 999/1.29M, 0/0       ├───────────────────────┘
│    trim/eff : 4.35%/2.65M, 99.58%                  │          [cpu009: 15%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer08} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 23 min, 5 sec      │  cycles done : 10    │
│   last new find : 0 days, 0 hrs, 1 min, 21 sec      │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 1 min, 28 sec      │saved crashes : 316   │
│ last saved hang : 0 days, 0 hrs, 2 min, 24 sec      │  saved hangs : 81    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 11.4k.103 (92.8%)  │    map density : 0.70% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 700 (5.70%)         │
│ stage execs : 130/300 (43.33%)       │  new edges on : 1063 (8.66%)        │
│ total execs : 49.5M                  │ total crashes : 3335 (316 saved)    │
│  exec speed : 17.5k/sec              │  total tmouts : 850 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 15/341k, 3/341k, 0/341k              │    levels : 20        │
│  byte flips : 0/42.7k, 4/42.6k, 0/42.6k            │   pending : 3         │
│ arithmetics : 5/2.98M, 0/5.95M, 0/5.94M            │  pend fav : 0         │
│  known ints : 0/383k, 0/1.62M, 0/2.38M             │ own finds : 2188      │
│  dictionary : 0/0, 0/0, 7/66.7M, 0/66.7M           │  imported : 1703      │
│havoc/splice : 1185/15.7M, 157/29.4M                │ stability : 98.29%    │
│py/custom/rq : unused, unused, 966/1.34M, 0/0       ├───────────────────────┘
│    trim/eff : 4.14%/2.61M, 99.70%                  │          [cpu010: 14%]                                                                                                                                           
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer09} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 55 min, 23 sec     │  cycles done : 6     │
│   last new find : 0 days, 0 hrs, 0 min, 7 sec       │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 0 min, 53 sec      │saved crashes : 237   │
│ last saved hang : 0 days, 0 hrs, 0 min, 23 sec      │  saved hangs : 62    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 8751.67 (71.3%)    │    map density : 0.33% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 696 (5.67%)         │
│ stage execs : 58/150 (38.67%)        │  new edges on : 1072 (8.73%)        │
│ total execs : 26.9M                  │ total crashes : 1950 (237 saved)    │
│  exec speed : 18.6k/sec              │  total tmouts : 171 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/8016, 0/8014, 0/8010               │    levels : 5         │
│  byte flips : 0/1002, 0/1000, 0/996                │   pending : 3         │
│ arithmetics : 0/70.1k, 0/139k, 0/139k              │  pend fav : 0         │
│  known ints : 0/9010, 0/38.0k, 0/55.8k             │ own finds : 277       │
│  dictionary : 0/0, 0/0, 0/880k, 0/881k             │  imported : 3614      │
│havoc/splice : 231/7.83M, 68/15.3M                  │ stability : 98.24%    │
│py/custom/rq : unused, unused, 199/845k, 0/0        ├───────────────────────┘
│    trim/eff : 3.89%/2.61M, 99.80%                  │          [cpu011: 10%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer10} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 55 min, 8 sec      │  cycles done : 6     │
│   last new find : 0 days, 0 hrs, 0 min, 9 sec       │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 0 min, 48 sec      │saved crashes : 242   │
│ last saved hang : 0 days, 0 hrs, 0 min, 50 sec      │  saved hangs : 54    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 12.3k.1 (100.0%)   │    map density : 0.74% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : quick eff              │ favored items : 695 (5.65%)         │
│ stage execs : 6544/65.5k (9.99%)     │  new edges on : 1072 (8.71%)        │
│ total execs : 26.3M                  │ total crashes : 2006 (242 saved)    │
│  exec speed : 367.8/sec              │  total tmouts : 168 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/102k, 0/102k, 0/102k               │    levels : 4         │
│  byte flips : 0/12.8k, 0/12.8k, 0/12.8k            │   pending : 2         │
│ arithmetics : 0/896k, 0/1.79M, 0/1.79M             │  pend fav : 1         │
│  known ints : 0/115k, 0/486k, 0/716k               │ own finds : 229       │
│  dictionary : 0/0, 0/0, 0/24.0M, 0/24.0M           │  imported : 3693      │
│havoc/splice : 225/7.63M, 21/14.9M                  │ stability : 98.29%    │
│py/custom/rq : unused, unused, 198/864k, 0/0        ├───────────────────────┘
│    trim/eff : 3.89%/2.61M, 99.98%                  │          [cpu012: 14%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer11} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 54 min, 54 sec     │  cycles done : 6     │
│   last new find : 0 days, 0 hrs, 0 min, 3 sec       │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 1 min, 17 sec      │saved crashes : 229   │
│ last saved hang : 0 days, 0 hrs, 0 min, 27 sec      │  saved hangs : 55    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 9277.89 (75.5%)    │    map density : 1.29% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.73 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 1               │ favored items : 694 (5.65%)         │
│ stage execs : 7/18 (38.89%)          │  new edges on : 1078 (8.77%)        │
│ total execs : 27.2M                  │ total crashes : 1910 (229 saved)    │
│  exec speed : 18.1k/sec              │  total tmouts : 178 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/47.9k, 0/47.9k, 0/47.9k            │    levels : 4         │
│  byte flips : 0/5984, 0/5983, 0/5981               │   pending : 6         │
│ arithmetics : 0/418k, 0/837k, 0/837k               │  pend fav : 0         │
│  known ints : 0/53.9k, 0/227k, 0/334k              │ own finds : 218       │
│  dictionary : 0/0, 0/0, 0/11.2M, 0/11.2M           │  imported : 3684      │
│havoc/splice : 221/7.99M, 14/15.6M                  │ stability : 98.31%    │
│py/custom/rq : unused, unused, 193/844k, 0/0        ├───────────────────────┘
│    trim/eff : 3.93%/2.57M, 99.98%                  │          [cpu013:  9%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

     AFL ++4.21c {fuzzer12} (../../target/debug/ssa_afl_fuzzer) [explore]     
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 54 min, 48 sec     │  cycles done : 5     │
│   last new find : 0 days, 0 hrs, 2 min, 41 sec      │ corpus count : 12.3k │
│last saved crash : 0 days, 0 hrs, 1 min, 8 sec       │saved crashes : 240   │
│ last saved hang : 0 days, 0 hrs, 0 min, 1 sec       │  saved hangs : 61    │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 10.0k.56 (81.5%)   │    map density : 1.58% / 3.75%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 4.74 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 3               │ favored items : 692 (5.63%)         │
│ stage execs : 4/14 (28.57%)          │  new edges on : 1078 (8.77%)        │
│ total execs : 25.5M                  │ total crashes : 1887 (240 saved)    │
│  exec speed : 20.0k/sec              │  total tmouts : 217 (0 saved)       │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/48.2k, 0/48.2k, 0/48.2k            │    levels : 4         │
│  byte flips : 0/6029, 0/6028, 0/6026               │   pending : 1         │
│ arithmetics : 0/421k, 0/842k, 0/842k               │  pend fav : 0         │
│  known ints : 0/54.2k, 0/228k, 0/337k              │ own finds : 223       │
│  dictionary : 0/0, 0/0, 0/12.9M, 0/12.9M           │  imported : 3690      │
│havoc/splice : 223/7.37M, 16/14.4M                  │ stability : 98.24%    │
│py/custom/rq : unused, unused, 195/830k, 0/0        ├───────────────────────┘
│    trim/eff : 3.85%/2.61M, 99.88%                  │          [cpu014:  7%]
└─ strategy: explore ────────── state: in progress ──┘^C

+++ Testing aborted by user +++



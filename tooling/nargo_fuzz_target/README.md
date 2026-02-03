# `nargo_fuzz_target`

To set up the inputs, empty the `draft_inputs` directory, if present, and run:

```bash
./populate_inputs.rb
```

To copy all `.nr` files with up to `512` bytes to the `draft_inputs` folder.

Then, copy any relevant inputs (e.g. all of them) to `inputs` and run with:

```bash
# to install 'cargo-afl'
cargo install cargo-afl

cargo afl build
cargo afl fuzz -i inputs -o outputs ../../target/debug/nargo_fuzz_target
```

## Collecting results:

```bash
# (from this directory)
./collect_unique_crashes.rb
```

To only collect outputs and deduplicate more manually, run:

```ruby
# ❯ ./collect_outputs.rb ./collected_outputs

def ff(x)
  `bash -c "cat #{x} | ../../../target/release/nargo compile --debug-compile-stdin 2>&1"`
end

def gg(x)
  puts('-'*80)
  puts ff(x)
  puts('-'*80)
end

# from ./collected_outputs/
Dir['./*'].map{|x| ff(x)}.sort.uniq.each{|x|puts('-'*80);puts(x);puts('-'*80)}
```

## Ramdisk

Using a ramdisk for the `inputs` and `outputs` folder may improve performance:

```bash
# make dir
❯ sudo mkdir /tmp/nargo_fuzz_ramdisk

# make accessible to other users
❯ sudo chmod 777 /tmp/nargo_fuzz_ramdisk

# now, the nargo binary is ~400MB
❯ ls -lah ../../target/debug/nargo_fuzz_target
-rwxrwxr-x 2 michael michael 398M May  1 11:32 ../../target/debug/nargo_fuzz_target

# so we round up to the next power of 2:
❯ sudo mount -t tmpfs -o size=512m nargo_fuzz_ramdisk /tmp/nargo_fuzz_ramdisk

# check for successful mount
❯ mount | tail -n 1
nargo_fuzz_ramdisk on /tmp/nargo_fuzz_ramdisk type tmpfs (rw,relatime,size=524288k,inode64)

# copy inputs/outputs to the ramdisk
❯ ..

# CLEANUP
# delete the ramdisk
sudo umount /tmp/nargo_fuzz_ramdisk/
```


## Example Results

Simple run in debug mode:

```
    AFL ++4.21c {default} (../../target/debug/nargo_fuzz_target) [explore]    
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 7 hrs, 54 min, 6 sec      │  cycles done : 0     │
│   last new find : 0 days, 0 hrs, 0 min, 55 sec      │ corpus count : 3361  │
│last saved crash : 0 days, 0 hrs, 6 min, 33 sec      │saved crashes : 30    │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1980*1 (58.9%)     │    map density : 0.84% / 1.14%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.84 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : havoc                  │ favored items : 267 (7.94%)         │
│ stage execs : 13/80 (16.25%)         │  new edges on : 483 (14.37%)        │
│ total execs : 734k                   │ total crashes : 34 (30 saved)       │
│  exec speed : 20.23/sec (slow!)      │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 3/7336, 3/7333, 2/7327               │    levels : 36        │
│  byte flips : 0/917, 0/914, 0/908                  │   pending : 2487      │
│ arithmetics : 9/64.1k, 0/127k, 0/126k              │  pend fav : 0         │
│  known ints : 0/8235, 1/34.7k, 2/50.8k             │ own finds : 3352      │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 0         │
│havoc/splice : 1145/163k, 1917/265k                 │ stability : 95.95%    │
│py/custom/rq : unused, unused, 282/175k, 0/0        ├───────────────────────┘
│    trim/eff : 0.18%/75.8k, 99.35%                  │          [cpu000: 18%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

[!] Stopped during the first cycle, results may be incomplete.
```

Simple run in release mode:

```
   AFL ++4.21c {default} (../../target/release/nargo_fuzz_target) [explore]   
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 7 hrs, 49 min, 31 sec     │  cycles done : 0     │
│   last new find : 0 days, 0 hrs, 0 min, 4 sec       │ corpus count : 3089  │
│last saved crash : 0 days, 0 hrs, 2 min, 52 sec      │saved crashes : 70    │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 3068.1 (99.3%)     │    map density : 0.87% / 1.17%      │
│  runs timed out : 1 (0.03%)          │ count coverage : 2.66 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : trim 16/16             │ favored items : 289 (9.36%)         │
│ stage execs : 2/17 (11.76%)          │  new edges on : 538 (17.42%)        │
│ total execs : 684k                   │ total crashes : 87 (70 saved)       │
│  exec speed : 29.48/sec (slow!)      │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 11/8688, 2/8684, 1/8676              │    levels : 23        │
│  byte flips : 0/1086, 2/1082, 0/1074               │   pending : 2402      │
│ arithmetics : 16/75.8k, 0/149k, 0/148k             │  pend fav : 7         │
│  known ints : 0/9728, 4/41.0k, 2/60.0k             │ own finds : 3080      │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 0         │
│havoc/splice : 719/116k, 2052/218k                  │ stability : 96.04%    │
│py/custom/rq : unused, unused, 320/222k, 0/0        ├───────────────────────┘
│    trim/eff : 0.01%/77.4k, 98.80%                  │          [cpu001: 20%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

[!] Stopped during the first cycle, results may be incomplete.
```

Simple run in release mode (ramdisk):

```
  AFL ++4.21c {default} (/tmp/nargo_fuzz_ramdisk/nargo_fuzz_target) [explore]  
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 1 hrs, 2 min, 55 sec      │  cycles done : 0     │
│   last new find : 0 days, 0 hrs, 0 min, 1 sec       │ corpus count : 1350  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 27.1 (2.0%)        │    map density : 0.83% / 1.04%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.15 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : splice 8               │ favored items : 157 (11.63%)        │
│ stage execs : 773/1536 (50.33%)      │  new edges on : 257 (19.04%)        │
│ total execs : 100k                   │ total crashes : 0 (0 saved)         │
│  exec speed : 29.59/sec (slow!)      │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 7/144, 2/143, 2/141                  │    levels : 10        │
│  byte flips : 0/18, 1/17, 0/15                     │   pending : 1298      │
│ arithmetics : 14/1189, 0/1707, 0/1540              │  pend fav : 123       │
│  known ints : 0/142, 3/595, 2/800                  │ own finds : 1341      │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 0         │
│havoc/splice : 129/7660, 1097/54.0k                 │ stability : 95.57%    │
│py/custom/rq : unused, unused, 51/16.0k, 0/0        ├───────────────────────┘
│    trim/eff : 0.11%/3471, 72.22%                   │          [cpu002: 15%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

[!] Stopped during the first cycle, results may be incomplete.
```

Independent simple release mode run before setting up proper parallel run:

```
   AFL ++4.21c {fuzzer3} (../../target/release/nargo_fuzz_target) [explore]   
┌─ process timing ────────────────────────────────────┬─ overall results ────┐
│        run time : 0 days, 0 hrs, 54 min, 48 sec     │  cycles done : 0     │
│   last new find : 0 days, 0 hrs, 0 min, 23 sec      │ corpus count : 1788  │
│last saved crash : none seen yet                     │saved crashes : 0     │
│ last saved hang : none seen yet                     │  saved hangs : 0     │
├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
│  now processing : 1637.1 (91.6%)     │    map density : 0.85% / 1.06%      │
│  runs timed out : 0 (0.00%)          │ count coverage : 2.27 bits/tuple    │
├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
│  now trying : colorization           │ favored items : 206 (11.52%)        │
│ stage execs : 496/500 (99.20%)       │  new edges on : 329 (18.40%)        │
│ total execs : 84.7k                  │ total crashes : 0 (0 saved)         │
│  exec speed : 19.07/sec (zzzz...)    │  total tmouts : 0 (0 saved)         │
├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
│   bit flips : 0/0, 0/0, 0/0                        │    levels : 4         │
│  byte flips : 0/0, 0/0, 0/0                        │   pending : 1754      │
│ arithmetics : 0/0, 0/0, 0/0                        │  pend fav : 185       │
│  known ints : 0/0, 0/0, 0/0                        │ own finds : 1280      │
│  dictionary : 0/0, 0/0, 0/0, 0/0                   │  imported : 499       │
│havoc/splice : 101/2064, 1135/50.8k                 │ stability : 95.62%    │
│py/custom/rq : unused, unused, 36/5243, 0/0         ├───────────────────────┘
│    trim/eff : 1.09%/1079, n/a                      │          [cpu005: 48%]
└─ strategy: explore ────────── state: in progress ──┘^C
                                                                                                                                                                                                                        
+++ Testing aborted by user +++

[!] Stopped during the first cycle, results may be incomplete.
```


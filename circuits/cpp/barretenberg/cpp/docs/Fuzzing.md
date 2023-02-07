# Fuzzing barretenberg
## Intro
We are gradually introducing fuzzing of various primitives into barretenberg, focusing first and foremost on in-cicruit types. If you are developing / patching a primitive and there is a fuzzer available for it, please take the time to update the fuzzer (if you've added new functionality) and run it for at least a few hours to increase security.

## Build

To build with standard clang:
```
mkdir build-fuzzing && cd build-fuzzing
cmake -DFUZZING=ON ..
make
```
Fuzzing build turns off building tests and benchmarks, since they are incompatible with libfuzzer interface.

To turn on address sanitizer add `-DADDRESS_SANITIZER=ON`. Note that address sanitizer can be used to explore crashes.
Sometimes you might have to specify the address of llvm-symbolizer. You have to do it with `export ASAN_SYMBOLIZER_PATH=<PATH_TO_SYMBOLIZER>`.
For undefined behaviour sanitizer `-DUNDEFINED_BEHAVIOUR_SANITIZER=ON`.
Note that the fuzzer can be orders of magnitude slower with ASan (2-3x slower) or UBSan on, so it is best to run a non-sanitized build first, minimize the testcase and then run it for a bit of time with sanitizers.

Building with clang 13 or later is recommended, since libfuzzer contains and by default utilizes the entropic power schedule, which is considered more efficient
than the standard one present in previous versions.
You can downloadload the latest clang+llvm release here: https://github.com/llvm/llvm-project/releases

To set up cmake with another version of clang and fuzzing on:

```bash
cmake -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_C_COMPILER=<path to clang> -DCMAKE_CXX_COMPILER=<path to clang++> -DFUZZING=ON  ..
```

## Currently supported
Currently we have fuzzers for bigfield, bit_array, bool, byte_array, field, safe_uint and uint. Each of them is available in 3 versions: StandardPlonk, TurboPlonk, ALL (differential fuzzing of 2 versions).
To compile all fuzzers just type `make`.
## Running the fuzzer
TODO: add information about saved testcases

You can simply run a fuzzer by executing the built executable, for example,
```bash
./bin/stdlib_primitives_bigfield_turbo_fuzzer
```
This is useful if you added a feature/instruction or changed some logic and want to quickly test if there are any really bad bugs.
To run the fuzzer seriously, I'd recommend:
```bash
mkdir ../../../<fuzzer_type>_testcases;
mkdir crashes;
./bin/<fuzzer_executable> -timeout=1 -len_control=500 -workers=8 -jobs=8 -entropic=1 -shrink=1 -artifact_prefix=crashes/ -use_value_profile=1 ../../../<fuzzer_type>_testcases
```
You can watch the progress of the fuzzer in one of the generated logs fuzz-<number>.log
The purpose of each parameter:
+ -timeout=1 - If a testcase takes more than 1 second to execute, it will be treated as a crash
+ -len_control=500 - Slows down the increase of testcase size. Especially important for heavy classes like bigfield, keeps the number of executions per second at a decent rate
+ -worker=8 - The number of threads that can simultaneously execute testcases. Should be less or equal to the number of jobs
+ -jobs=8 - After how many crashes the fuzzer will stop fuzzing. If a crash is executed and the number of jobs is more than workers then the fuzzer will proceed to give the worker a new job. The 8/8 worker/job configuration ensures that the fuzzer will quit after 8 crashes and until the first crash all the workers are busy.
+ -entropic=1 - Entropic should be enabled by default, but in case it isn't, enable it. A better power schedule than the old one.
+ -shrink=1 - If a new testcase is encountered that has the same coverage as some previous one in the corpus and the testcase is smaller, replace the one in the corpus with the new one. Helps keep exec/s higher.
+ -artifact_prefix=crashes/ - Where to save crashes/timeouts/ooms.
+ -use_value_profile=1 - Leverage libfuzzer internal CMP analysis. Very useful, but blows the corpus up.
+ <PATH_TO_CORPUS> (../../../<fuzzer_type>_testcases) - The path to the folder, where corpus testcases are going to be saved and loaded from (also loads testcases from there at the start of fuzzing).

Log structure is described here   https://llvm.org/docs/LibFuzzer.html

If you've found an issue, stopped the fuzzer, you can minimize the corpus to get rid of repetitions and then start from a minimized corpus

```bash
mkdir ../../../<fuzzer_type>_testcases_minimized;
./bin/<fuzzer_executable> -merge=1 -use_value_profile=1  ../../../<fuzzer_type>_testcases_minimized ../../../<fuzzer_type>_testcases;
rm  ../../../<fuzzer_type>_testcases/*;
cp ../../../<fuzzer_type>_testcases_minimized/* ../../../<fuzzer_type>_testcases;
```

If you've found a crash, you can minimize the crash to make the root cause more obvious:
```bash
mkdir minimized_crashes
./bin/<fuzzer_executable> -minimize_crash=1 -artifact_prefix=minimized_crashes <crash_file>
```
Also, both bigfield and safeuint fuzzer containt the SHOW_INFORMATION preprocessor cases, which enable the printing of instructions and values to make debugging the crash easier.

# Coverage reports

Build with coverage instrumentation:

```cpp
mkdir build-coverage/
cd build-coverage/
cmake -DFUZZING=ON -DCMAKE_CXX_FLAGS="-fprofile-instr-generate -fcoverage-mapping" ..
cmake --build . --parallel
```

Then run the fuzzer on the corpus and generate the HTML coverage reports:

```
LLVM_PROFILE_FILE="coverage.profraw" ./bin/<fuzzer> corpus/ -runs=1
llvm-profdata merge -sparse coverage.profraw -o coverage.profdata
llvm-cov show -output-dir=out/report -format=html ./bin/<fuzzer> -instr-profile=coverage.profdata
```

View the coverage reports with your web browser:

```
python3 -m http.server --directory out/
```

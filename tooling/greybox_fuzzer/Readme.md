# Noir Greybox Fuzzer
This is a Rust implementation of a greybox fuzzer, which is a type of testing tool that automatically generates and mutates test inputs to find bugs in programs. This fuzzer in particular, can automatically generate test cases for Noir programs with very little effort from the program writer. Here's a high-level overview of the key components:

## Core Fuzzing Structure

The main struct FuzzedExecutor manages the fuzzing process
It executes programs in two modes: ACIR (Arithmetic Circuit Intermediate Representation) and Brillig (unconstrained execution mode). Both modes are used in tandem to leverage different coverage mechanisms. It uses parallel execution with multiple threads to improve performance.

## Key Features
- Uses coverage-guided fuzzing (tracks which parts of the program are executed)
- Maintains a corpus of test cases that provide good coverage
- Uses mutation-based fuzzing (modifies existing inputs to create new test cases and can use crossover)
- Can detect discrepancies between ACIR and Brillig execution modes
- Can explore failing programs and detect only specific failure conditions
- Includes performance metrics and pretty printing of progress
- Supports corpus minimization (finding a smaller set of inputs that maintain coverage), although for now only the lazy approach is implemented
- Can be used with an oracle to perform differential fuzzing against a known good implementation in another language

## Usage

A simple example of a fuzzing harness is the following:

```noir
#[fuzz]
fn fuzz_add(a: Field, b: Field) {
    assert(a!=(b+3));
}
```

Given a noir program, with a fuzzing harness, the fuzzer can be run with the following command:

```bash
nargo fuzz [FUZZING_HARNESS_NAME]
```

If [FUZZING_HARNESS_NAME] is given, only the fuzzing harnesses with names containing this string will be run.

Additional fuzzing-specific options include:

      --corpus-dir <CORPUS_DIR>
          If given, load/store fuzzer corpus from this folder
      --minimized-corpus-dir <MINIMIZED_CORPUS_DIR>
          If given, perform corpus minimization instead of fuzzing and store results in the given folder
      --fuzzing-failure-dir <FUZZING_FAILURE_DIR>
          If given, store the failing input in the given folder
      --vector-all
          Vector all available harnesses that match the name (doesn't perform any fuzzing)
      --num-threads <NUM_THREADS>
          The number of threads to use for fuzzing [default: 1]
      --exact
          Only run harnesses that match exactly
      --timeout <TIMEOUT>
          Maximum time in seconds to spend fuzzing per harness (default: no timeout)

`--show-output` and `--oracle-resolver` can be used in the same way as with regular execution and testing.
It is recommended to use `--skip-underconstrained-check` to increase compilation speed.

## Fuzzing more complex programs

A lot of the time, the program will already have many expected assertions that would lead to a failing test case, for example:

```noir
#[fuzz]
fn fuzz_add(a: u64, b: u64) {
    assert((a+b-15)!=(a-b+30));
}
```
Using integer arithmetic will often automatically lead to overflows and underflows, which will lead to a failing test case. To avoid this we can specify and "only_fail_with" attribute to the fuzzing harness, which will only mark a testcase as failing if the assertion contains a specific message:

```noir
#[fuzz(only_fail_with = "This is the message that will be checked for")]
fn fuzz_add(a: u64, b: u64) {
    assert((a+b-15)!=(a-b+30), "This is the message that will be checked for");
}
```
N.B. You can't find a failing testcase for this specific example, as the assertion is always true.

## Fuzzing Output

When running a fuzzing harness, you'll see output similar to this:

![Fuzzing progress stream showing stats like executions per second, coverage, and crashes found](fuzzing_stream.png)
The output is streamed to the console, and shows key metrics like:
- CNT: Number of test cases executed
- CRPS: Number of active test cases in the corpus
- AB_NEW: Number of test cases added to the corpus with ACIR and Brillig
- B_NEW: Number of test cases added to the corpus with Brillig 
- RMVD: Number of test cases removed from the corpus
- A_TIME: Time spent fuzzing with ACIR (cumulative from all threads)
- B_TIME: Time spent fuzzing with Brillig (cumulative from all threads)
- M_TIME: Time spend mutating test cases (cumulative from all threads)
- RND_SIZE: Size of the last ROUND (number of test cases)
- RND_EX_TIME: How much time was spent executing the test cases in the last round (in a single thread)
- UPD_TIME: Time spent updating the corpus (in a single thread)

If there is new coverage found, the fuzzer will print "NEW:" at the beginning of the line, else it will print "LOOP:"

Let's change the assertion in the example above to:
```noir
#[fuzz(only_fail_with = "This is the message that will be checked for")]
fn fuzz_add(a: u64, b: u64) {
    assert((a+b-16)!=(a-b+30), "This is the message that will be checked for");
}
```
Now, when we run the fuzzer, we'll see the following output:

![Fuzzing failure output showing the failing test case and its inputs](fuzzing_failure.png)

It quickly finds a failing test case, and prints the inputs that lead to the failure in the json format. It also shows the path of the file where it saves a failing toml file, which will have the name "Prover-failling-\<package-name\>-\<fuzzing-harness-name\>.toml".


## Potential improvements (in order of most to least impact)

- Implement partial witness generation for ACIR (https://github.com/noir-lang/noir/issues/7502)
- Implement a full oracle for fuzzing (allowing to execute brillig and get the boolean result and then determine failure based on that) (https://github.com/noir-lang/noir/issues/7447)
- Implement initial witness level splicing (https://github.com/noir-lang/noir/issues/7503)
- Implement Mutation Optimizations (automatically detect efficient mutations) (https://github.com/noir-lang/noir/issues/7446)
- Implement proper minimization of the corpus (where we first detect inputs with unique features and then join them with a minimal set of other inputs to have full coverage) (https://github.com/noir-lang/noir/issues/7448)
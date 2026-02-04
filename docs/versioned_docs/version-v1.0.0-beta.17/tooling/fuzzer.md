---
title: Fuzzer
description: The Noir Fuzzer is a tool that allows you to fuzz your Noir programs.
keywords: [Fuzzing, Noir, Noir Fuzzer, Security]
sidebar_position: 3
---

The Noir Fuzzer is a tool that allows you to fuzz your Noir programs. It is a type of testing tool that automatically generates and mutates test inputs to find bugs in programs. This fuzzer in particular, can automatically generate test cases for Noir programs with very little effort from the program writer.

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

```rust
#[fuzz]
fn fuzz_add(a: Field, b: Field) {
    assert(a != (b+3));
}
```

Given a noir program, with a fuzzing harness, the fuzzer can be run with the following command:

```bash
nargo fuzz [FUZZING_HARNESS_NAME]
```

If [FUZZING_HARNESS_NAME] is given, only the fuzzing harnesses with names containing this string will be run.

An example of the output of the fuzzer is shown below:

![Basic fuzzer example](@site/static/img/tooling/fuzzer/basic-fuzzer-example.png)

By default, the fuzzer will save the corpus for a particular harness in the `corpus/<package_name>/<harness_name>` directory, but this can be changed by specifying the `--corpus-dir <DIR>` option, which will save the corpus in `<DIR>/<package_name>/<harness_name>`.

The fuzzer will output metrics about the fuzzing process, when new coverage is discovered (starting with `NEW`) or if some time has passed since the last output (`LOOP`). The time since the last output is doubled every `LOOP`, unless new coverage is discovered. Then it is reset to the initial interval of 1 second.


The output is streamed to the console, and shows key metrics like:
- `CNT` - Number of test cases executed
- `CRPS` - Number of active test cases in the corpus
- `AB_NEW` - Number of test cases added to the corpus with ACIR/Brillig hybrid execution
- `B_NEW` - Number of test cases added to the corpus with Brillig
- `RMVD` - Number of test cases removed from the corpus, because other test cases have been found that have the same coverage
- `A_TIME` - Time spent fuzzing with ACIR (cumulative from all threads)
- `B_TIME` - Time spent fuzzing with Brillig (cumulative from all threads)
- `M_TIME` - Time spend mutating test cases (cumulative from all threads)
- `RND_SIZE` - Number of mutated test cases in the last round (round is until new coverage is discovered or 100ms has passed)
- `RND_EX_TIME` - How much time was spent executing the test cases in the last round (in a single thread)
- `UPD_TIME` - Time spent updating the corpus (in a single thread)

If the timeout is not specified, the fuzzer will run until it finds a failing test case. By default the failing test case is saved in the `Prover-failing-<package_name>-<harness_name>.toml` file. So that it can be easily used with nargo execute by renaming it to `Prover.toml` and renaming the harness to `main`.

Additional fuzzing-specific options include:

      --corpus-dir <CORPUS_DIR>
          If given, load/store fuzzer corpus from this folder
      --minimized-corpus-dir <MINIMIZED_CORPUS_DIR>
          If given, perform corpus minimization instead of fuzzing and store results in the given folder
      --fuzzing-failure-dir <FUZZING_FAILURE_DIR>
          If given, store the failing input in the given folder
      --list-all
          List all available harnesses that match the name (doesn't perform any fuzzing)
      --num-threads <NUM_THREADS>
          The number of threads to use for fuzzing [default: 1]
      --exact
          Only run harnesses that match exactly
      --timeout <TIMEOUT>
          Maximum time in seconds to spend fuzzing per harness (default: no timeout)
      --max-executions <MAX_EXECUTIONS>
          Maximum number of executions of ACIR and Brillig per harness (default: no limit)

`--show-output` and `--oracle-resolver` can be used in the same way as with regular execution and testing.
It is recommended to use `--skip-underconstrained-check` to increase compilation speed.


## Fuzzing more complex programs
### Using `should_fail` and `should_fail_with`

The fuzzer can be used to fuzz programs that are expected to fail. To do this, you can use the `should_fail` and `should_fail_with` attributes.

The following example will fuzz the program with the `should_fail` attribute, and will only consider a test case as a failure if the program passes:
```rust
#[fuzz(should_fail)]
fn fuzz_should_fail(a: [bool; 32]) {
    let mut or_sum= false;
    for i in 0..32 {
        or_sum=or_sum|(a[i]==((i&1)as bool));
    }
    assert(!or_sum);
}
```

The `should_fail_with` expects that the program will fail with a specific error message. The following example will fuzz the program with the `should_fail_with` attribute, and will only consider a test case as a failure if the program passes or fails with the message different from "This is the message that will be checked for":
```rust
#[fuzz(should_fail_with = "This is the message that will be checked for")]
fn fuzz_should_fail_with(a: [bool; 32]) {
    let mut or_sum= false;
    for i in 0..32 {
        or_sum=or_sum|(a[i]==((i&1)as bool));
    }
    assert(or_sum);
    assert(false, "This is the message that will be checked for");
}
```

### Using `only_fail_with`
A lot of the time, the program will already have many expected assertions that would lead to a failing test case, for example:

```rust
#[fuzz]
fn fuzz_add(a: u64, b: u64) {
    assert((a+b-15)!=(a-b+30));
}
```
Using integer arithmetic will often automatically lead to overflows and underflows, which will lead to a failing test case. If we want to check that a specific property is broken, rather than detect all failures, we can specify an "only_fail_with" attribute to the fuzzing harness, which will only mark a testcase as failing if the assertion contains a specific message:

```rust
#[fuzz(only_fail_with = "This is the message that will be checked for")]
fn fuzz_add(a: u64, b: u64) {
    assert((a+b-15)!=(a-b+30), "This is the message that will be checked for");
}
```
N.B. You can't find a failing testcase for this specific example, as the assertion is always true.

Let's change the assertion in the example above to:
```rust
#[fuzz(only_fail_with = "This is the message that will be checked for")]
fn fuzz_add(a: u64, b: u64) {
    assert((a+b-16)!=(a-b+30), "This is the message that will be checked for");
}
```
Now, when we run the fuzzer, we'll see the following output:

![Fuzzing failure output showing the failing test case and its inputs](@site/static/img/tooling/fuzzer/only-fail-with-example.png)

### Using an oracle

You can use an oracle to perform differential fuzzing against a known good implementation in another language. To do this you need to specify an oracle in the code, and run the fuzzer with the `--oracle-resolver <ORACLE_RESOLVER_URL>` option.

For this example, we'll use the following noir program:
```rust
#[oracle(check_addition)]
unconstrained fn check_addition(a: u32, b: u32, c: u32) -> bool {}
unconstrained fn check_addition_wrapper(a: u32, b: u32, c: u32) -> bool {
    check_addition(a, b, c)
}

#[fuzz(only_fail_with = "addition incorrect")]
fn main(a: u32, b: u32) {
    let c = a + b + ((b - a == 49)  as u32);
    // Safety: this is for fuzzing purposes only
    assert(unsafe { check_addition_wrapper(a, b, c) }, "addition incorrect");
}
```
You can create a simple python server to resolve the oracle (you'll have to install `werkzeug` and `jsonrpc` through your chosen package manager):

```python
from werkzeug.serving import run_simple
from werkzeug.wrappers import Response, Request

from jsonrpc import JSONRPCResponseManager, dispatcher

@dispatcher.add_method
def resolve_foreign_call(arg):
    assert arg["function"]=="check_addition"
    a=int(arg["inputs"][0],16)
    b=int(arg["inputs"][1],16)
    c=int(arg["inputs"][2],16)
    success=(a+b==c)
    result=dict()
    result['values']=["1" if success else "0"]
    return result


@Request.application
def application(request):
    response = JSONRPCResponseManager.handle(
        request.data, dispatcher)
    return Response(response.json, mimetype='application/json')

if __name__ == '__main__':
    run_simple('localhost', 40000, application)
```
You need to run this server before running the fuzzer.

Now if you run the fuzzer, you can see the following output:

![Fuzzing failure output showing oracle-checked failure](@site/static/img/tooling/fuzzer/oracle-fuzzing.png)

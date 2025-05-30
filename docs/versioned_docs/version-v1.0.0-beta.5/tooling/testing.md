---
title: Testing in Noir
description: Learn how to use Nargo to test your Noir program in a quick and easy way
keywords: [Nargo, testing, Noir, compile, test]
sidebar_position: 1
---

You can test your Noir programs using Noir circuits.

Nargo will automatically compile and run any functions which have the decorator `#[test]` on them if
you run `nargo test`.

For example if you have a program like:

```rust
fn add(x: u64, y: u64) -> u64 {
    x + y
}
#[test]
fn test_add() {
    assert(add(2,2) == 4);
    assert(add(0,1) == 1);
    assert(add(1,0) == 1);
}
```

Running `nargo test` will test that the `test_add` function can be executed while satisfying all
the constraints which allows you to test that add returns the expected values. Test functions can't
have any arguments currently.

### Test fail

You can write tests that are expected to fail by using the decorator `#[test(should_fail)]`. For example:

```rust
fn add(x: u64, y: u64) -> u64 {
    x + y
}
#[test(should_fail)]
fn test_add() {
    assert(add(2,2) == 5);
}
```

You can be more specific and make it fail with a specific reason by using `should_fail_with = "<the reason for failure>"`:

```rust
fn main(african_swallow_avg_speed : Field) {
    assert(african_swallow_avg_speed == 65, "What is the airspeed velocity of an unladen swallow");
}

#[test]
fn test_king_arthur() {
    main(65);
}

#[test(should_fail_with = "What is the airspeed velocity of an unladen swallow")]
fn test_bridgekeeper() {
    main(32);
}
```

The string given to `should_fail_with` doesn't need to exactly match the failure reason, it just needs to be a substring of it:

```rust
fn main(african_swallow_avg_speed : Field) {
    assert(african_swallow_avg_speed == 65, "What is the airspeed velocity of an unladen swallow");
}

#[test]
fn test_king_arthur() {
    main(65);
}

#[test(should_fail_with = "airspeed velocity")]
fn test_bridgekeeper() {
    main(32);
}
```

### Fuzz tests

You can write fuzzing harnesses that will run on `nargo test` by using the decorator `#[test]` with a function that has arguments. For example:

```rust
#[test]
fn test_basic(a: Field, b: Field) {
    assert(a + b == b + a);
}
```
The test above is not expected to fail. By default, the fuzzer will run for 1 second and use 100000 executions (whichever comes first). All available threads will be used for each fuzz test.
The fuzz tests also work with `#[test(should_fail)]` and `#[test(should_fail_with = "<the reason for failure>")]`. For example:

```rust
#[test(should_fail)]
fn test_should_fail(a: [bool; 32]) {
    let mut or_sum= false;
    for i in 0..32 {
        or_sum=or_sum|(a[i]==((i&1)as bool));
    }
    assert(!or_sum);
}
```
or

```rust
#[test(should_fail_with = "This is the message that will be checked for")]
fn fuzz_should_fail_with(a: [bool; 32]) {
    let mut or_sum= false;
    for i in 0..32 {
        or_sum=or_sum|(a[i]==((i&1)as bool));
    }
    assert(or_sum);
    assert(false, "This is the message that will be checked for");
}
```

The underlying fuzzing mechanism is described in the [Fuzzer](../tooling/fuzzer) documentation.

There are some fuzzing-specific options that can be used with `nargo test`:
       --no-fuzz
          Do not run fuzz tests (tests that have arguments)

      --only-fuzz
          Only run fuzz tests (tests that have arguments)

      --corpus-dir <CORPUS_DIR>
          If given, load/store fuzzer corpus from this folder

      --minimized-corpus-dir <MINIMIZED_CORPUS_DIR>
          If given, perform corpus minimization instead of fuzzing and store results in the given folder

      --fuzzing-failure-dir <FUZZING_FAILURE_DIR>
          If given, store the failing input in the given folder

      --fuzz-timeout <FUZZ_TIMEOUT>
          Maximum time in seconds to spend fuzzing (default: 1 second)

          [default: 1]

      --fuzz-max-executions <FUZZ_MAX_EXECUTIONS>
          Maximum number of executions to run for each fuzz test (default: 100000)

          [default: 100000]

      --fuzz-show-progress
          Show progress of fuzzing (default: false)


By default, the fuzzing corpus is saved in a temporary directory, but this can be changed. This allows you to resume fuzzing from the same corpus if the process is interrupted, if you want to run continuous fuzzing on your corpus, or if you want to use previous failures for regression testing.


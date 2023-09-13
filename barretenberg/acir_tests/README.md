# Acir Test Vector Runner

The aim is to verify acir tests verify through a given backend binary. "Backend binaries" can include e.g.:

- bb (native CLI)
- bb.js (typescript CLI)
- bb.js-dev (symlink in your PATH that runs the typescript CLI via ts-node)
- bb.js.browser (script in `headless-test` that runs a a test through bb.js in a browser instance via playwright)

To run:

```
$ ./run_acir_tests.sh
```

This will clone the acir test vectors from the noir repo, and will iterate over each one, running it through the
`../cpp/build/bin/bb` binary (by default) `prove_and_verify` command.

You can substitute the backend binary using the `BIN` environment variable.
You can turn on logging with `VERBOSE` environment variable.
You can specify a specific test to run.

```
$ BIN=bb.js VERBOSE=1 ./run_acir_tests.sh 1_mul
```

You can use a relative path to an executable. e.g. if bb.js-dev is not symlinked into your PATH:

```
$ BIN=../ts/bb.js-dev VERBOSE=1 ./run_acir_tests.sh 1_mul
```

```
$ BIN=./headless-test/bb.js.browser VERBOSE=1 ./run_acir_tests.sh 1_mul
```

You can specify a different testing "flow" with with `FLOW` environment variable. Flows are in the `flows` dir.
The default flow is `prove_and_verify`, which is the quickest way to... prove and verify. It's used to test the acir
test vectors actually all pass in whichever version of the backend is being run.
The `all_cmds` flow tests all the supported commands on the binary. Slower, but is there to test the cli.

```
$ FLOW=all_cmds ./run_acir_tests.sh 1_mul
```

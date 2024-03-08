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

## Regenerating witness for `double_verify_proof` and `double_verify_nested_proof`

`double_verify_proof` has inputs that are proof system specific such as the circuit verification key and the proofs themselves which are being recursively verified. Certain proof system changes can sometimes lead to the key or inner proofs now being invalid.

This means we have to generate the proof specific inputs using our backend and pass it back into `double_verify_proof` to regenerate the accurate witness. The following is a temporary solution to manually regenerate the inputs for `double_verify_proof` on a specific Noir branch.

First find `acir_tests/gen_inner_proof_inputs.sh`. Change the $BRANCH env var to your working branch and $PROOF_NAME to your first input you want to recursively verify. The script is going to generate the proof system specific verification key output and proof for the `assert_statement_recursive` test. 

To run:
```
./gen_inner_proof_inputs.sh
```
To generate a new input you can run the script again. To generate a new file under `assert_statement_recursive/proofs/` be sure to change the $PROOF_NAME inside of the script.

You can then copy these inputs over to your working branch in Noir and regenerate the witness for `double_verify_proof`. You can then change the branch in `run_acir_tests.sh` to this Noir working branch as well and `double_verify_proof` should pass.

The same process should then be repeated, but now `double_verify_proof_recursive` will be the circuit for which we will be generating recursive inputs using `gen_inner_proof_inputs.sh`. The recursive artifacts should then supplied as inputs to `double_verify_nested_proof`. 
# Noir Minimizer

The minimizer is an experimental tool with the primary goal to help reduce the size of Noir examples found
by the AST fuzzer that fail due to some reason, and require minimization before a bug ticket can be opened.

It relies on [cvise](https://github.com/marxin/cvise?tab=readme-ov-file) to try and make the Noir program
smaller, while preserving some error message we receive from `nargo`.

The tool requires [Docker](https://docs.docker.com/engine/install/) being installed on the developer machine.

## Usage

### Build a Docker image

`cvise` has binaries published for Linux; in the interest of being cross platform, we build a Docker image
that contains `cvise` and `nargo`, which it repeatedly invokes to compile the Noir code.

Execute the following command to build the image. This needs to be done every time `nargo` itself changes:

```shell
tooling/ast_fuzzer/minimizer/scripts/docker-build.sh
```

### Minimize Noir

To minimize a Noir project, it has to be in a single `main.nr` file, with a corresponding `Prover.toml` file,
and we need a single line of the error message we are looking to preserve from its original output.

With that, we need to invoke the `minimize.sh` script as follows:

```shell
tooling/ast_fuzzer/minimizer/scripts/minimize.sh "<error-message>" execute [compile options...] path/to/main.nr
```

The command can be `compile` or `execute`. The latter needs a `Prover.toml` file, which can be given following
the path to `main.nr`, or it is assumed to be in the parent directory of `main.nr`, like a regular Noir project.

You can also supply `nargo` compilation options. This is useful for cases where we want to minimize a program
that only fails under certain compilation conditions (e.g., experimental features, different compilation pipeline). Example usage:

```shell
tooling/ast_fuzzer/minimizer/scripts/minimize.sh "<error-message>" execute -Zenums --minimal-ssa path/to/main.nr
```

The script makes a `main.nr.bkp` backup file, because the tool will minimize the Noir code in-place.

## Examples

> Note that the errors in this examples hopefully will have been fixed. They are here for illustration only.

### Assertion failure in `execute`

Say we have this code, which found by the fuzzer for https://github.com/noir-lang/noir/issues/8803

<details>
<summary>Problematic `main.nr` found by the fuzzer</summary>

```rust
global G_A: [bool; 3] = [false, true, true];
global G_B: bool = false;
global G_C: Field = -144409342013671434790742305428920231458;
unconstrained fn main() -> return_data i32 {
    let mut ctx_limit: u32 = 25;
    if func_1((&mut ["IKA", "ALO", "OIL"]), (&mut ctx_limit))[(2072760302 % 3)] {
        let mut a: [(u128, str<3>, i32, u128); 4] = [
            (
                if ((!G_A[1]) <= func_1((&mut ["HFQ", "QPY", "WQC"]), (&mut ctx_limit))[0]) {
                    if G_B {
                        if true {
                            (G_C as u128)
                        } else {
                            if true {
                                (G_C as u128)
                            } else {
                                if G_B {
                                    (G_C as u128)
                                } else {
                                    if true {
                                        if func_1((&mut ["VDK", "MSE", "XBE"]), (&mut ctx_limit))[1]
                                             {
                                                if func_1(
                                                    (&mut ["WPK", "DIS", "AEH"]),
                                                    (&mut ctx_limit),
                                                )[2] {
                                                    (G_C as u128)
                                                } else {
                                                    (G_C as u128)
                                                }
                                            } else {
                                                (G_C as u128)
                                            }
                                        } else {
                                            (G_C as u128)
                                        }
                                    }
                                }
                            }
                        } else {
                            (G_C as u128)
                        }
                    } else {
                        (G_C as u128)
                    }, "GQD", 33713394, (G_C as u128),
                ),
                ((G_C as u128), "ZLA", 847084415, (G_C as u128)),
                ((G_C as u128), "FCA", -2071434514, (G_C as u128)),
                ((G_C as u128), "ITC", -148063243, (G_C as u128)),
            ];
            -1807850365
        } else {
            1598311787
        }
    }
unconstrained fn func_1(a: &mut [str<3>; 3], _ctx_limit: &mut u32) -> [bool; 3] {
    let i: &mut [str<3>; 3] = {
        let mut b: &mut bool = (&mut true);
        b = b;
        b = b;
        {
            let mut idx_c: u32 = 0;
            while ((*b) <= G_A[0]) {
                if (idx_c == 1) {
                    break
                } else {
                    idx_c = (idx_c + 1);
                    for idx_d in 1373182677..1373182678 {
                        {
                            let mut idx_e: u32 = 0;
                            loop {
                                if (idx_e == 7) {
                                    break
                                } else {
                                    idx_e = (idx_e + 1);
                                    break;
                                    b = b;
                                    {
                                        let mut idx_f: u32 = 0;
                                        loop {
                                            if (idx_f == 9) {
                                                break
                                            } else {
                                                idx_f = (idx_f + 1);
                                                break;
                                                let h = {
                                                    {
                                                        let mut idx_g: u32 = 0;
                                                        while (idx_d >= idx_d) {
                                                            if (idx_g == 5) {
                                                                break
                                                            } else {
                                                                idx_g = (idx_g + 1);
                                                                b = b;
                                                            }
                                                        }
                                                    };
                                                    (G_B as Field)
                                                };
                                                break;
                                            }
                                        }
                                    };
                                }
                            }
                        };
                        break;
                    }
                }
            }
        };
        (&mut ["NXB", "YLT", "FQU"])
    };
    G_A
}
```

</details>

When we try to execute, we get this error:

```console
❯ cargo run -q -p nargo_cli -- execute --silence-warnings

error: Assertion failed: 'Bit size for rhs 254 does not match op bit size 1'
   ┌─ src/main.nr:22:16
   │
22 │         while ((*b) <= G_A[0]) {}
   │                --------------
   │
   = Call stack:
     1. src/main.nr:5:8
     2. src/main.nr:22:16

Failed assertion
```

That's pretty clear, but the code is a bit large. See if we can reduce it.

First try with a related, but different error message, just to see what happens:

```console
❯ tooling/ast_fuzzer/minimizer/scripts/scripts/minimize.sh "condition value is not a boolean: MismatchedBitSize" execute test_programs/execution_success/fuzz_testing/src/main.nr
C-Vise cannot run because the interestingness test does not return
zero. Please ensure that it does so not only in the directory where
you are invoking C-Vise, but also in an arbitrary temporary
directory containing only the files that are being reduced. In other
words, running these commands:

  DIR=`mktemp -d`
  cp /noir/main.nr $DIR
  cd $DIR
  /noir/check.sh
  echo $?

should result in '0' being echoed to the terminal.
Please ensure that the test script takes no arguments; it should be hard-coded to refer
to the same file that is passed as an argument to C-Vise.
```

So `cvise` is telling us that the script the tool prepared did not return 0, which is because it looked for an error message that did not appear in the output.

Try again, with the correct message:

```console
❯ tooling/ast_fuzzer/minimizer/scripts/minimize.sh "Bit size for rhs 254 does not match op bit size 1" execute test_programs/execution_success/fuzz_testing/src/main.nr
00:00:00 INFO ===< 9 >===
00:00:00 INFO running 14 interestingness tests in parallel
00:00:00 INFO INITIAL PASSES
00:00:00 INFO ===< BlankPass >===
00:00:00 INFO ===< LinesPass::0 >===
00:00:00 INFO ===< LinesPass::1 >===
00:00:01 INFO ===< LinesPass::2 >===
00:00:02 INFO (-0.6%, 5173 bytes, 25 lines)
00:00:02 INFO ===< LinesPass::3 >===
00:00:03 INFO ===< LinesPass::4 >===
...
00:00:55 INFO (86.6%, 687 bytes, 29 lines)
00:00:55 INFO (86.7%, 682 bytes, 29 lines)
00:00:55 INFO (87.0%, 668 bytes, 28 lines)
00:00:56 INFO (87.1%, 666 bytes, 27 lines)
00:00:57 INFO (87.1%, 661 bytes, 27 lines)
00:01:31 INFO Exiting now ...
```

We can observe the changes it makes by keeping `main.nr` open in our editor. After about a minute the file stops changing, and we can stop the tool with `Ctrl+C`.

The end result is much smaller:

<details>
<summary>`main.nr` minimized by `cvise`</summary>

```rust
global G_A: [bool] = [true];
global G_B: bool = false;
unconstrained fn main() -> return_data i32 {
    let mut ctx_limit = 25;
    if func_1((&mut ["IKA", "ALO", "OIL"]), (&mut ctx_limit))[0] {
        [
            (
                if (func_1((&mut ["HFQ", "QPY", "WQC"]), (&mut ctx_limit))[0]) {
                    0
                } else {
                    0
                }
            ),
        ];
    }
    1598311787
}
unconstrained fn func_1(a: &mut [str<3>; 3], _ctx_limit: &mut u32) -> [bool] {
    {
        let mut b = (&mut true);
        b = b;
        while ((*b) <= G_A[0]) {}
    };
    G_A
}
```

</details>

It got rid of a lot of cruft, which allows us to focus on what matters:

<details>
<summary>Final `main.nr` further minimized by hand</summary>

```rust
unconstrained fn main() {
    let mut b: &mut bool = (&mut true);
    b = b;
    {
        while ((*b) <= false) {}
    };
}
```

</details>

### Crash in `compile`

Another example is the code found by the fuzzer in https://github.com/noir-lang/noir/issues/8741

The original code looked as follows:

<details>
<summary>Problematic `main.nr` found by the fuzzer</summary>

```rust
global G_A: bool = false;
fn main(a: pub [(bool, bool, str<3>, bool); 4], b: (bool, bool, str<3>, bool), c: str<4>) -> return_data bool {
    let mut ctx_limit: u32 = 25;
    let i = unsafe { func_2_proxy(ctx_limit) };
    let h = if b.0 {
        let d = b.2;
        let mut g = if b.3 {
            let f: [&mut [&mut bool; 2]; 2] = {
                let mut e: &mut bool = (&mut false);
                e = {
                    e = e;
                    if b.3 {
                        e = (&mut false);
                        e = e;
                        e = e;
                        e = e;
                    };
                    e
                };
                [(&mut [(&mut true), (&mut false)]), (&mut [(&mut false), (&mut true)])]
            };
            if false {
                a
            } else {
                a
            }
        } else {
            a
        };
        c
    } else {
        c
    };
    b.0
}
fn func_1(ctx_limit: &mut u32) -> bool {
    if ((*ctx_limit) == 0) {
        true
    } else {
        *ctx_limit = ((*ctx_limit) - 1);
        let mut h: str<4> = {
            let mut a = (unsafe { func_2_proxy((*ctx_limit)) }.1 as Field);
            for idx_b in 48 .. 48 {
                for idx_c in 37919 .. 37925 {
                    a = (unsafe { func_2_proxy((*ctx_limit)) }.1 as Field);
                };
                for idx_d in 17890133749029059494 .. 17890133749029059501 {
                    for idx_e in 2936542607166930997 .. 2936542607166930989 {
                        a = (((idx_d as Field) * (G_A as Field)) / (unsafe { func_2_proxy((*ctx_limit)) }.0 as Field));
                        let g: &mut bool = {
                            let mut f: Field = {
                                308424986754900546907368585881390441546
                            };
                            (&mut true)
                        };
                        a = (-(G_A as Field));
                    };
                    a = (-(idx_d as Field));
                };
                a = -216918869032603336751134960482740787067;
            };
            "OULD"
        };
        true
    }
}
unconstrained fn func_2(ctx_limit: &mut u32) -> (bool, bool, str<3>, bool) {
    if ((*ctx_limit) == 0) {
        (false, true, "SUY", false)
    } else {
        *ctx_limit = ((*ctx_limit) - 1);
        func_2(ctx_limit)
    }
}
unconstrained fn func_2_proxy(mut ctx_limit: u32) -> (bool, bool, str<3>, bool) {
    func_2((&mut ctx_limit))
}
```

</details>

It consists of 4 functions. Trying to compile it crashed the compiler:

```console
❯ cargo run -q -p nargo_cli -- compile
The application panicked (crashed).
Message:  Cannot return references from an if expression
Location: compiler/noirc_evaluator/src/ssa/opt/flatten_cfg/value_merger.rs:68
```

See if we can minimize it:

```console
❯ tooling/ast_fuzzer/minimizer/scripts/minimize.sh "Cannot return references from an if expression" execute $PWD/test_programs/execution_success/fuzz_testing/src/main.nr
00:00:00 INFO ===< 10 >===
00:00:00 INFO running 14 interestingness tests in parallel
00:00:00 INFO INITIAL PASSES
00:00:00 INFO ===< BlankPass >===
00:00:00 INFO ===< LinesPass::0 >===
00:00:00 INFO (48.2%, 1320 bytes, 5 lines)
...
00:00:53 INFO ===< IndentPass::final >===
00:00:53 INFO (94.8%, 133 bytes, 5 lines)
00:00:53 INFO ===================== done ====================
===< PASS statistics >===
  pass name                                              time (s) time (%)   worked   failed  total executed
  ClexPass::rm-tok-pattern-4                                 7.88    14.63        9      894            1041
  ...
  IntsPass::d                                                0.00     0.00        0        0               0

Runtime: 54 seconds
Reduced test-cases:

--- /noir/main.nr ---
fn main(b : (bool, bool, str<3>, bool))->return_data bool {
  let mut e = &mut false;
  e = { if b .3 {e = &mut false} e };
  b .0
}
```

That's pretty much spot on, almost the same as the one in the ticket!
---
title: Nargo
description:
  Noir CLI Commands for Noir Prover and Verifier to create, execute, prove and verify programs,
  generate Solidity verifier smart contract and compile into JSON file containing ACIR
  representation and ABI of circuit.
keywords:
  [
    Nargo,
    Noir CLI,
    Noir Prover,
    Noir Verifier,
    generate Solidity verifier,
    compile JSON file,
    ACIR representation,
    ABI of circuit,
    TypeScript,
  ]
sidebar_position: 0
---

# Command-Line Help for `nargo`

This document contains the help content for the `nargo` command-line program.

**Command Overview:**

* [`nargo`↴](#nargo)
* [`nargo check`↴](#nargo-check)
* [`nargo fmt`↴](#nargo-fmt)
* [`nargo compile`↴](#nargo-compile)
* [`nargo new`↴](#nargo-new)
* [`nargo init`↴](#nargo-init)
* [`nargo execute`↴](#nargo-execute)
* [`nargo debug`↴](#nargo-debug)
* [`nargo test`↴](#nargo-test)
* [`nargo info`↴](#nargo-info)
* [`nargo lsp`↴](#nargo-lsp)
* [`nargo generate-completion-script`↴](#nargo-generate-completion-script)

## `nargo`

Noir's package manager

**Usage:** `nargo <COMMAND>`

###### **Subcommands:**

* `check` — Checks the constraint system for errors
* `fmt` — Format the Noir files in a workspace
* `compile` — Compile the program and its secret execution trace into ACIR format
* `new` — Create a Noir project in a new directory
* `init` — Create a Noir project in the current directory
* `execute` — Executes a circuit to calculate its return value
* `debug` — Executes a circuit in debug mode
* `test` — Run the tests for this program
* `info` — Provides detailed information on each of a program's function (represented by a single circuit)
* `lsp` — Starts the Noir LSP server
* `generate-completion-script` — Generates a shell completion script for your favorite shell

###### **Options:**




## `nargo check`

Checks the constraint system for errors

**Usage:** `nargo check [OPTIONS]`

###### **Options:**

* `--package <PACKAGE>` — The name of the package to run the command on. By default run on the first one found moving up along the ancestors of the current directory
* `--workspace` — Run on all packages in the workspace

  Possible values: `true`, `false`

* `--overwrite` — Force overwrite of existing files

  Possible values: `true`, `false`

* `--expression-width <EXPRESSION_WIDTH>` — Specify the backend expression width that should be targeted
* `--bounded-codegen` — Generate ACIR with the target backend expression width. The default is to generate ACIR without a bound and split expressions after code generation. Activating this flag can sometimes provide optimizations for certain programs

  Default value: `false`

  Possible values: `true`, `false`

* `--force` — Force a full recompilation

  Possible values: `true`, `false`

* `--print-acir` — Display the ACIR for compiled circuit

  Possible values: `true`, `false`

* `--deny-warnings` — Treat all warnings as errors

  Possible values: `true`, `false`

* `--silence-warnings` — Suppress warnings

  Possible values: `true`, `false`

* `--debug-comptime-in-file <DEBUG_COMPTIME_IN_FILE>` — Enable printing results of comptime evaluation: provide a path suffix for the module to debug, e.g. "package_name/src/main.nr"
* `--skip-underconstrained-check` — Flag to turn off the compiler check for under constrained values. Warning: This can improve compilation speed but can also lead to correctness errors. This check should always be run on production code

  Possible values: `true`, `false`




## `nargo fmt`

Format the Noir files in a workspace

**Usage:** `nargo fmt [OPTIONS]`

###### **Options:**

* `--check` — Run noirfmt in check mode

  Possible values: `true`, `false`

* `--package <PACKAGE>` — The name of the package to run the command on. By default run on the first one found moving up along the ancestors of the current directory
* `--workspace` — Run on all packages in the workspace

  Possible values: `true`, `false`




## `nargo compile`

Compile the program and its secret execution trace into ACIR format

**Usage:** `nargo compile [OPTIONS]`

###### **Options:**

* `--package <PACKAGE>` — The name of the package to run the command on. By default run on the first one found moving up along the ancestors of the current directory
* `--workspace` — Run on all packages in the workspace

  Possible values: `true`, `false`

* `--expression-width <EXPRESSION_WIDTH>` — Specify the backend expression width that should be targeted
* `--bounded-codegen` — Generate ACIR with the target backend expression width. The default is to generate ACIR without a bound and split expressions after code generation. Activating this flag can sometimes provide optimizations for certain programs

  Default value: `false`

  Possible values: `true`, `false`

* `--force` — Force a full recompilation

  Possible values: `true`, `false`

* `--print-acir` — Display the ACIR for compiled circuit

  Possible values: `true`, `false`

* `--deny-warnings` — Treat all warnings as errors

  Possible values: `true`, `false`

* `--silence-warnings` — Suppress warnings

  Possible values: `true`, `false`

* `--debug-comptime-in-file <DEBUG_COMPTIME_IN_FILE>` — Enable printing results of comptime evaluation: provide a path suffix for the module to debug, e.g. "package_name/src/main.nr"
* `--skip-underconstrained-check` — Flag to turn off the compiler check for under constrained values. Warning: This can improve compilation speed but can also lead to correctness errors. This check should always be run on production code

  Possible values: `true`, `false`




## `nargo new`

Create a Noir project in a new directory

**Usage:** `nargo new [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>` — The path to save the new project

###### **Options:**

* `--name <NAME>` — Name of the package [default: package directory name]
* `--lib` — Use a library template

  Possible values: `true`, `false`

* `--bin` — Use a binary template [default]

  Possible values: `true`, `false`

* `--contract` — Use a contract template

  Possible values: `true`, `false`




## `nargo init`

Create a Noir project in the current directory

**Usage:** `nargo init [OPTIONS]`

###### **Options:**

* `--name <NAME>` — Name of the package [default: current directory name]
* `--lib` — Use a library template

  Possible values: `true`, `false`

* `--bin` — Use a binary template [default]

  Possible values: `true`, `false`

* `--contract` — Use a contract template

  Possible values: `true`, `false`




## `nargo execute`

Executes a circuit to calculate its return value

**Usage:** `nargo execute [OPTIONS] [WITNESS_NAME]`

###### **Arguments:**

* `<WITNESS_NAME>` — Write the execution witness to named file

Defaults to the name of the package being executed.

###### **Options:**

* `-p`, `--prover-name <PROVER_NAME>` — The name of the toml file which contains the inputs for the prover

  Default value: `Prover`
* `--package <PACKAGE>` — The name of the package to run the command on. By default run on the first one found moving up along the ancestors of the current directory
* `--workspace` — Run on all packages in the workspace

  Possible values: `true`, `false`

* `--expression-width <EXPRESSION_WIDTH>` — Specify the backend expression width that should be targeted
* `--bounded-codegen` — Generate ACIR with the target backend expression width. The default is to generate ACIR without a bound and split expressions after code generation. Activating this flag can sometimes provide optimizations for certain programs

  Default value: `false`

  Possible values: `true`, `false`

* `--force` — Force a full recompilation

  Possible values: `true`, `false`

* `--print-acir` — Display the ACIR for compiled circuit

  Possible values: `true`, `false`

* `--deny-warnings` — Treat all warnings as errors

  Possible values: `true`, `false`

* `--silence-warnings` — Suppress warnings

  Possible values: `true`, `false`

* `--debug-comptime-in-file <DEBUG_COMPTIME_IN_FILE>` — Enable printing results of comptime evaluation: provide a path suffix for the module to debug, e.g. "package_name/src/main.nr"
* `--skip-underconstrained-check` — Flag to turn off the compiler check for under constrained values. Warning: This can improve compilation speed but can also lead to correctness errors. This check should always be run on production code

  Possible values: `true`, `false`

* `--oracle-resolver <ORACLE_RESOLVER>` — JSON RPC url to solve oracle calls



## `nargo debug`

Executes a circuit in debug mode

**Usage:** `nargo debug [OPTIONS] [WITNESS_NAME]`

###### **Arguments:**

* `<WITNESS_NAME>` — Write the execution witness to named file

###### **Options:**

* `-p`, `--prover-name <PROVER_NAME>` — The name of the toml file which contains the inputs for the prover

  Default value: `Prover`
* `--package <PACKAGE>` — The name of the package to execute
* `--expression-width <EXPRESSION_WIDTH>` — Specify the backend expression width that should be targeted
* `--bounded-codegen` — Generate ACIR with the target backend expression width. The default is to generate ACIR without a bound and split expressions after code generation. Activating this flag can sometimes provide optimizations for certain programs

  Default value: `false`

  Possible values: `true`, `false`

* `--force` — Force a full recompilation

  Possible values: `true`, `false`

* `--print-acir` — Display the ACIR for compiled circuit

  Possible values: `true`, `false`

* `--deny-warnings` — Treat all warnings as errors

  Possible values: `true`, `false`

* `--silence-warnings` — Suppress warnings

  Possible values: `true`, `false`

* `--debug-comptime-in-file <DEBUG_COMPTIME_IN_FILE>` — Enable printing results of comptime evaluation: provide a path suffix for the module to debug, e.g. "package_name/src/main.nr"
* `--skip-underconstrained-check` — Flag to turn off the compiler check for under constrained values. Warning: This can improve compilation speed but can also lead to correctness errors. This check should always be run on production code

  Possible values: `true`, `false`

* `--acir-mode` — Force ACIR output (disabling instrumentation)

  Possible values: `true`, `false`

* `--skip-instrumentation <SKIP_INSTRUMENTATION>` — Disable vars debug instrumentation (enabled by default)

  Possible values: `true`, `false`




## `nargo test`

Run the tests for this program

**Usage:** `nargo test [OPTIONS] [TEST_NAME]`

###### **Arguments:**

* `<TEST_NAME>` — If given, only tests with names containing this string will be run

###### **Options:**

* `--show-output` — Display output of `println` statements

  Possible values: `true`, `false`

* `--exact` — Only run tests that match exactly

  Possible values: `true`, `false`

* `--package <PACKAGE>` — The name of the package to run the command on. By default run on the first one found moving up along the ancestors of the current directory
* `--workspace` — Run on all packages in the workspace

  Possible values: `true`, `false`

* `--expression-width <EXPRESSION_WIDTH>` — Specify the backend expression width that should be targeted
* `--bounded-codegen` — Generate ACIR with the target backend expression width. The default is to generate ACIR without a bound and split expressions after code generation. Activating this flag can sometimes provide optimizations for certain programs

  Default value: `false`

  Possible values: `true`, `false`

* `--force` — Force a full recompilation

  Possible values: `true`, `false`

* `--print-acir` — Display the ACIR for compiled circuit

  Possible values: `true`, `false`

* `--deny-warnings` — Treat all warnings as errors

  Possible values: `true`, `false`

* `--silence-warnings` — Suppress warnings

  Possible values: `true`, `false`

* `--debug-comptime-in-file <DEBUG_COMPTIME_IN_FILE>` — Enable printing results of comptime evaluation: provide a path suffix for the module to debug, e.g. "package_name/src/main.nr"
* `--skip-underconstrained-check` — Flag to turn off the compiler check for under constrained values. Warning: This can improve compilation speed but can also lead to correctness errors. This check should always be run on production code

  Possible values: `true`, `false`

* `--oracle-resolver <ORACLE_RESOLVER>` — JSON RPC url to solve oracle calls



## `nargo info`

Provides detailed information on each of a program's function (represented by a single circuit)

Current information provided per circuit: 1. The number of ACIR opcodes 2. Counts the final number gates in the circuit used by a backend

**Usage:** `nargo info [OPTIONS]`

###### **Options:**

* `--package <PACKAGE>` — The name of the package to run the command on. By default run on the first one found moving up along the ancestors of the current directory
* `--workspace` — Run on all packages in the workspace

  Possible values: `true`, `false`

* `--profile-execution`

  Possible values: `true`, `false`

* `-p`, `--prover-name <PROVER_NAME>` — The name of the toml file which contains the inputs for the prover

  Default value: `Prover`
* `--expression-width <EXPRESSION_WIDTH>` — Specify the backend expression width that should be targeted
* `--bounded-codegen` — Generate ACIR with the target backend expression width. The default is to generate ACIR without a bound and split expressions after code generation. Activating this flag can sometimes provide optimizations for certain programs

  Default value: `false`

  Possible values: `true`, `false`

* `--force` — Force a full recompilation

  Possible values: `true`, `false`

* `--print-acir` — Display the ACIR for compiled circuit

  Possible values: `true`, `false`

* `--deny-warnings` — Treat all warnings as errors

  Possible values: `true`, `false`

* `--silence-warnings` — Suppress warnings

  Possible values: `true`, `false`

* `--debug-comptime-in-file <DEBUG_COMPTIME_IN_FILE>` — Enable printing results of comptime evaluation: provide a path suffix for the module to debug, e.g. "package_name/src/main.nr"
* `--skip-underconstrained-check` — Flag to turn off the compiler check for under constrained values. Warning: This can improve compilation speed but can also lead to correctness errors. This check should always be run on production code

  Possible values: `true`, `false`




## `nargo lsp`

Starts the Noir LSP server

Starts an LSP server which allows IDEs such as VS Code to display diagnostics in Noir source.

VS Code Noir Language Support: https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir

**Usage:** `nargo lsp`



## `nargo generate-completion-script`

Generates a shell completion script for your favorite shell

**Usage:** `nargo generate-completion-script <SHELL>`

###### **Arguments:**

* `<SHELL>` — The shell to generate completions for. One of: bash, elvish, fish, powershell, zsh



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>


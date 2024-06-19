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
* [`nargo test`↴](#nargo-test)
* [`nargo info`↴](#nargo-info)
* [`nargo lsp`↴](#nargo-lsp)

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
* `test` — Run the tests for this program
* `info` — Provides detailed information on each of a program's function (represented by a single circuit)
* `lsp` — Starts the Noir LSP server

###### **Options:**




## `nargo check`

Checks the constraint system for errors

**Usage:** `nargo check [OPTIONS]`

###### **Options:**

* `--package <PACKAGE>` — The name of the package to check
* `--workspace` — Check all packages in the workspace
* `--overwrite` — Force overwrite of existing files
* `--expression-width <EXPRESSION_WIDTH>` — Override the expression width requested by the backend

  Default value: `4`
* `--force` — Force a full recompilation
* `--print-acir` — Display the ACIR for compiled circuit
* `--deny-warnings` — Treat all warnings as errors
* `--silence-warnings` — Suppress warnings



## `nargo fmt`

Format the Noir files in a workspace

**Usage:** `nargo fmt [OPTIONS]`

###### **Options:**

* `--check` — Run noirfmt in check mode



## `nargo compile`

Compile the program and its secret execution trace into ACIR format

**Usage:** `nargo compile [OPTIONS]`

###### **Options:**

* `--package <PACKAGE>` — The name of the package to compile
* `--workspace` — Compile all packages in the workspace
* `--expression-width <EXPRESSION_WIDTH>` — Override the expression width requested by the backend

  Default value: `4`
* `--force` — Force a full recompilation
* `--print-acir` — Display the ACIR for compiled circuit
* `--deny-warnings` — Treat all warnings as errors
* `--silence-warnings` — Suppress warnings



## `nargo new`

Create a Noir project in a new directory

**Usage:** `nargo new [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>` — The path to save the new project

###### **Options:**

* `--name <NAME>` — Name of the package [default: package directory name]
* `--lib` — Use a library template
* `--bin` — Use a binary template [default]
* `--contract` — Use a contract template



## `nargo init`

Create a Noir project in the current directory

**Usage:** `nargo init [OPTIONS]`

###### **Options:**

* `--name <NAME>` — Name of the package [default: current directory name]
* `--lib` — Use a library template
* `--bin` — Use a binary template [default]
* `--contract` — Use a contract template



## `nargo execute`

Executes a circuit to calculate its return value

**Usage:** `nargo execute [OPTIONS] [WITNESS_NAME]`

###### **Arguments:**

* `<WITNESS_NAME>` — Write the execution witness to named file

###### **Options:**

* `-p`, `--prover-name <PROVER_NAME>` — The name of the toml file which contains the inputs for the prover

  Default value: `Prover`
* `--package <PACKAGE>` — The name of the package to execute
* `--workspace` — Execute all packages in the workspace
* `--expression-width <EXPRESSION_WIDTH>` — Override the expression width requested by the backend

  Default value: `4`
* `--force` — Force a full recompilation
* `--print-acir` — Display the ACIR for compiled circuit
* `--deny-warnings` — Treat all warnings as errors
* `--silence-warnings` — Suppress warnings
* `--oracle-resolver <ORACLE_RESOLVER>` — JSON RPC url to solve oracle calls



## `nargo test`

Run the tests for this program

**Usage:** `nargo test [OPTIONS] [TEST_NAME]`

###### **Arguments:**

* `<TEST_NAME>` — If given, only tests with names containing this string will be run

###### **Options:**

* `--show-output` — Display output of `println` statements
* `--exact` — Only run tests that match exactly
* `--package <PACKAGE>` — The name of the package to test
* `--workspace` — Test all packages in the workspace
* `--expression-width <EXPRESSION_WIDTH>` — Override the expression width requested by the backend

  Default value: `4`
* `--force` — Force a full recompilation
* `--print-acir` — Display the ACIR for compiled circuit
* `--deny-warnings` — Treat all warnings as errors
* `--silence-warnings` — Suppress warnings
* `--oracle-resolver <ORACLE_RESOLVER>` — JSON RPC url to solve oracle calls



## `nargo info`

Provides detailed information on each of a program's function (represented by a single circuit)

Current information provided per circuit: 1. The number of ACIR opcodes 2. Counts the final number gates in the circuit used by a backend

**Usage:** `nargo info [OPTIONS]`

###### **Options:**

* `--package <PACKAGE>` — The name of the package to detail
* `--workspace` — Detail all packages in the workspace
* `--expression-width <EXPRESSION_WIDTH>` — Override the expression width requested by the backend

  Default value: `4`
* `--force` — Force a full recompilation
* `--print-acir` — Display the ACIR for compiled circuit
* `--deny-warnings` — Treat all warnings as errors
* `--silence-warnings` — Suppress warnings



## `nargo lsp`

Starts the Noir LSP server

Starts an LSP server which allows IDEs such as VS Code to display diagnostics in Noir source.

VS Code Noir Language Support: https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir

**Usage:** `nargo lsp`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>


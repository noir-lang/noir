---
title: Commands
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
---

## General options

| Option               | Description                                        |
| -------------------- | -------------------------------------------------- |
| `--show-ssa`         | Emit debug information for the intermediate SSA IR |
| `--deny-warnings`    | Quit execution when warnings are emitted           |
| `--silence-warnings` | Suppress warnings                                  |
| `-h, --help`         | Print help                                         |

## `nargo help [subcommand]`

Prints the list of available commands or specific information of a subcommand.

_Arguments_

| Argument       | Description                                  |
| -------------- | -------------------------------------------- |
| `<subcommand>` | The subcommand whose help message to display |

## `nargo backend`

Installs and selects custom backends used to generate and verify proofs.

### Commands

| Command     | Description                                               |
| ----------- | --------------------------------------------------------- |
| `current`   | Prints the name of the currently active backend           |
| `ls`        | Prints the list of currently installed backends           |
| `use`       | Select the backend to use                                 |
| `install`   | Install a new backend from a URL                          |
| `uninstall` | Uninstalls a backend                                      |
| `help`      | Print this message or the help of the given subcommand(s) |

### Options

| Option       | Description |
| ------------ | ----------- |
| `-h, --help` | Print help  |

## `nargo check`

Generate the `Prover.toml` and `Verifier.toml` files for specifying prover and verifier in/output
values of the Noir program respectively.

### Options

| Option                | Description                           |
| --------------------- | ------------------------------------- |
| `--package <PACKAGE>` | The name of the package to check      |
| `--workspace`         | Check all packages in the workspace   |
| `--print-acir`        | Display the ACIR for compiled circuit |
| `--deny-warnings`     | Treat all warnings as errors          |
| `--silence-warnings`  | Suppress warnings                     |
| `-h, --help`          | Print help                            |

### `nargo codegen-verifier`

Generate a Solidity verifier smart contract for the program.

### Options

| Option                | Description                           |
| --------------------- | ------------------------------------- |
| `--package <PACKAGE>` | The name of the package to codegen    |
| `--workspace`         | Codegen all packages in the workspace |
| `--print-acir`        | Display the ACIR for compiled circuit |
| `--deny-warnings`     | Treat all warnings as errors          |
| `--silence-warnings`  | Suppress warnings                     |
| `-h, --help`          | Print help                            |

## `nargo compile`

Compile the program into a JSON build artifact file containing the ACIR representation and the ABI
of the circuit. This build artifact can then be used to generate and verify proofs.

You can also use "build" as an alias for compile (e.g. `nargo build`).

### Options

| Option                | Description                                                  |
| --------------------- | ------------------------------------------------------------ |
| `--include-keys`      | Include Proving and Verification keys in the build artifacts |
| `--package <PACKAGE>` | The name of the package to compile                           |
| `--workspace`         | Compile all packages in the workspace                        |
| `--print-acir`        | Display the ACIR for compiled circuit                        |
| `--deny-warnings`     | Treat all warnings as errors                                 |
| `--silence-warnings`  | Suppress warnings                                            |
| `-h, --help`          | Print help                                                   |

## `nargo new <PATH>`

Creates a new Noir project in a new folder.

**Arguments**

| Argument | Description                      |
| -------- | -------------------------------- |
| `<PATH>` | The path to save the new project |

### Options

| Option          | Description                                           |
| --------------- | ----------------------------------------------------- |
| `--name <NAME>` | Name of the package [default: package directory name] |
| `--lib`         | Use a library template                                |
| `--bin`         | Use a binary template [default]                       |
| `--contract`    | Use a contract template                               |
| `-h, --help`    | Print help                                            |

## `nargo init`

Creates a new Noir project in the current directory.

### Options

| Option          | Description                                           |
| --------------- | ----------------------------------------------------- |
| `--name <NAME>` | Name of the package [default: current directory name] |
| `--lib`         | Use a library template                                |
| `--bin`         | Use a binary template [default]                       |
| `--contract`    | Use a contract template                               |
| `-h, --help`    | Print help                                            |

## `nargo execute [WITNESS_NAME]`

Runs the Noir program and prints its return value.

**Arguments**

| Argument         | Description                               |
| ---------------- | ----------------------------------------- |
| `[WITNESS_NAME]` | Write the execution witness to named file |

### Options

| Option                            | Description                                                                          |
| --------------------------------- | ------------------------------------------------------------------------------------ |
| `-p, --prover-name <PROVER_NAME>` | The name of the toml file which contains the inputs for the prover [default: Prover] |
| `--package <PACKAGE>`             | The name of the package to execute                                                   |
| `--workspace`                     | Execute all packages in the workspace                                                |
| `--print-acir`                    | Display the ACIR for compiled circuit                                                |
| `--deny-warnings`                 | Treat all warnings as errors                                                         |
| `--silence-warnings`              | Suppress warnings                                                                    |
| `-h, --help`                      | Print help                                                                           |

_Usage_

The inputs to the circuit are read from the `Prover.toml` file generated by `nargo check`, which
must be filled in.

To save the witness to file, run the command with a value for the `WITNESS_NAME` argument. A
`<WITNESS_NAME>.tr` file will then be saved in the `./target` folder.

## `nargo prove`

Creates a proof for the program.

### Options

| Option                                | Description                                                                              |
| ------------------------------------- | ---------------------------------------------------------------------------------------- |
| `-p, --prover-name <PROVER_NAME>`     | The name of the toml file which contains the inputs for the prover [default: Prover]     |
| `-v, --verifier-name <VERIFIER_NAME>` | The name of the toml file which contains the inputs for the verifier [default: Verifier] |
| `--verify`                            | Verify proof after proving                                                               |
| `--package <PACKAGE>`                 | The name of the package to prove                                                         |
| `--workspace`                         | Prove all packages in the workspace                                                      |
| `--print-acir`                        | Display the ACIR for compiled circuit                                                    |
| `--deny-warnings`                     | Treat all warnings as errors                                                             |
| `--silence-warnings`                  | Suppress warnings                                                                        |
| `-h, --help`                          | Print help                                                                               |

## `nargo verify`

Given a proof and a program, verify whether the proof is valid.

### Options

| Option                                | Description                                                                              |
| ------------------------------------- | ---------------------------------------------------------------------------------------- |
| `-v, --verifier-name <VERIFIER_NAME>` | The name of the toml file which contains the inputs for the verifier [default: Verifier] |
| `--package <PACKAGE>`                 | The name of the package to verify                                                        |
| `--workspace`                         | Verify all packages in the workspace                                                     |
| `--print-acir`                        | Display the ACIR for compiled circuit                                                    |
| `--deny-warnings`                     | Treat all warnings as errors                                                             |
| `--silence-warnings`                  | Suppress warnings                                                                        |
| `-h, --help`                          | Print help                                                                               |

## `nargo test [TEST_NAME]`

Nargo will automatically compile and run any functions which have the decorator `#[test]` on them if
you run `nargo test`. To print `println` statements in tests, use the `--show-output` flag.

Takes an optional `--exact` flag which allows you to select tests based on an exact name.

See an example on the [testing page](./testing).

### Options

| Option                | Description                            |
| --------------------- | -------------------------------------- |
| `--show-output`       | Display output of `println` statements |
| `--exact`             | Only run tests that match exactly      |
| `--package <PACKAGE>` | The name of the package to test        |
| `--workspace`         | Test all packages in the workspace     |
| `--print-acir`        | Display the ACIR for compiled circuit  |
| `--deny-warnings`     | Treat all warnings as errors           |
| `--silence-warnings`  | Suppress warnings                      |
| `-h, --help`          | Print help                             |

## `nargo info`

Prints a table containing the information of the package.

Currently the table provide

1. The number of ACIR opcodes
2. The final number gates in the circuit used by a backend

If the file contains a contract the table will provide the
above information about each function of the contract.

## `nargo lsp`

Start a long-running Language Server process that communicates over stdin/stdout.
Usually this command is not run by a user, but instead will be run by a Language Client, such as [vscode-noir](https://github.com/noir-lang/vscode-noir).

## `nargo fmt`

Automatically formats your Noir source code based on the default formatting settings.

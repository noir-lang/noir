# Structure

Below we briefly describe the purpose of each crate related to the compiler in this repository.

## fm - File Manager

This is the abstraction that the compiler uses to manage source files.

## noirc_abi

When consuming input from the user, a common ABI must be provided such that input provided in JSON/TOML can be converted to noir data types. This crate defines such an ABI.

## noirc_errors

This crate defines how compiler errors are displayed in the users standard output.

## noirc_evaluator

This crate can be seen as the middle end. It is in charge of generating the ACIR, and thus can be seen as the circuit generation pass.

## noirc_frontend

This crate comprises of the first few compiler passes that together we denote as the compiler frontend (in order): lexing, parsing, name resolution, type checking, and monomorphization. If any of these passes error, the resulting monomorphized AST will not be passed to the middle-end (noirc_evaluator)

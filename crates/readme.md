# Structure

Below we briefly describe the purpose of each crate in this repository.

## acir - Abstract Circuit Intermediate Representation

This is the intermediate representation that Noir compiles down to. It is agnostic to any particular NP-Complete language.

## acvm - Abstract Circuit Virtual Machine

This is the virtual machine that runs ACIR. Given a proving system to power it, one can create and verify proofs, create smart contracts that verify proofs.

## fm - File Manager

This is the abstraction that the compiler uses to manage source files.

## nargo

This is the default package manager used by Noir. One may draw similarities to Rusts' Cargo.

## noir_field

Since the DSL allows a user to create constants which can be as large as the field size, we must have a datatype that is able to store them. This is the purpose of the field crate.
One could alternatively use a BigInt library and store the modulus in a Context struct that gets passed to every compiler pass.

## noirc_abi

When consuming input from the user, a common ABI must be provided such that input provided in JSON/TOML can be converted to noir data types. This crate defines such an ABI.

## noirc_errors

This crate defines how compiler errors are displayed in the users standard output.

## noirc_evaluator

This crate can be seen as the middle end. It is in charge of generating the ACIR, and thus can be seen as the circuit generation pass.

## noirc_frontend

This crate comprises of the first few compiler passes that together we denote as the compiler frontend (in order): lexing, parsing, name resolution, type checking, and monomorphization. If any of these passes error, the resulting monomorphized AST will not be passed to the middle-end (noirc_evaluator)

## wasm

This crate is used to compile the Noir compiler into wasm. This is useful in the context where one wants to compile noir programs in the web browser.
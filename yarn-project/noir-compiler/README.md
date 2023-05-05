# Aztec Noir compiler

The Aztec noir compiler compiles noir contracts using noir_wasm and outputs Aztec formatted contract ABIs.

## Installation

To install the package, just run `yarn add @aztec/noir-compiler`.

## Usage

To run the compiler as a CLI tool, run `yarn aztec_noir_compiler compile <path_to_noir_contract_crate>`

## Status

Currently, this noir compiler uses noir master branch. It's not compatible yet with the test contracts for Aztec that are in the `noir-contracts` package, that need to be built following its README.md instructions.

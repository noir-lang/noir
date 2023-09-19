---
title: Functions
---

## `constructor`

- A special `constructor` function MUST be declared within a contract's scope.
- A constructor doesn't have a name, because its purpose is clear: to initialise state.
- In Aztec terminology, a constructor is always a 'private function' (i.e. it cannot be an `open` function).
- A constructor behaves almost identically to any other function. It's just important for Aztec to be able to identify this function as special: it may only be called once, and will not be deployed as part of the contract.

## secret functions

> a.k.a. "private" functions

#include_code functions-SecretFunction /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust

## `open` functions

> a.k.a. "public" functions

#include_code functions-OpenFunction /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust

## `unconstrained` functions

#include_code functions-UncontrainedFunction /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust

## Visibility

### `internal`

Similar to Solidity, internal functions and vars can be accessed within the contract itself. While technically callable from other contracts, there is a dynamic check that validates the caller to ensure it's the same contract.

### `external`

External is not used explicitly as it is in Solidity, but things not marked as `internal` will be external.

### `#[aztec(public)]` and `#[aztec(private)]`

These are used to annotate functions so that they are compliant with Aztec ABIs. They inject `PublicContext` and `PrivateContext` for use in contracts.

---
title: How to write an initializer function
---

This page explains how to write an initializer function.

To learn more about initializers, read [this](./main.md#initializer-functions).

Initializers are regular functions that set an "initialized" flag (a nullifier) for the contract. A contract can only be initialized once, and contract functions can only be called after the contract has been initialized, much like a constructor. However, if a contract defines no initializers, it can be called at any time. Additionally, you can define as many initializer functions in a contract as you want, both private and public.

## Annotate with `#[aztec(private)]` and `#[aztec(initializer)]`

Define your initializer like so:

```rust
#[aztec(private)]
#[aztec(initializer)]
fn constructor(){
    // function logic here
}
```

## Initializer with logic

Initializers are commonly used to set an admin, such as this example:

#include_code constructor /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

Here, the initializer is calling a public function. It can also call a private function. Learn more about calling functions from functions [here](../functions/call_functions.md).

To see constructors in action, check out the [Aztec.nr getting started guide](../../../getting_started/aztecnr-getting-started.md).

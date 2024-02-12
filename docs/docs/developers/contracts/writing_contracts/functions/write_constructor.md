---
title: How to write a constructor
---

This page explains how to write a constructor function.

To learn more about constructors, read [this](./main.md#constructors).

## Annotate with `#[aztec(private)]`

Currently, all constructors in Aztec must be private.

Define your constructor like so:

```rust
#[aztec(private)]
fn constructor()
```

## Option 1: Empty constructor

Your constructor does not need to do anything; you can leave it blank like so:

```rust
#[aztec(private)]
fn constructor() {}
```

## Option 2: Constructor with logic

Constructors are commonly used to set an admin, such as this example:

#include_code constructor /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

Here, the constructor is calling a public function. It can also call a private function. Learn more about calling functions from functions [here](../functions/call_functions.md).

To see constructors in action, check out the [Aztec.nr getting started guide](../../../getting_started/aztecnr-getting-started.md).

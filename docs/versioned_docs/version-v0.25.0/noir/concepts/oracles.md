---
title: Oracles
description: Dive into how Noir supports Oracles via RPC calls, and learn how to declare an Oracle in Noir with our comprehensive guide.
keywords:
  - Noir
  - Oracles
  - RPC Calls
  - Unconstrained Functions
  - Programming
  - Blockchain
sidebar_position: 6
---

Noir has support for Oracles via RPC calls. This means Noir will make an RPC call and use the return value for proof generation.

Since Oracles are not resolved by Noir, they are [`unconstrained` functions](./unconstrained.md)

You can declare an Oracle through the `#[oracle(<name>)]` flag. Example:

```rust
#[oracle(get_number_sequence)]
unconstrained fn get_number_sequence(_size: Field) -> [Field] {}
```

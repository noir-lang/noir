---
title: How to define contract storage
---

On this page, you will learn how to define storage in your smart contract.

To learn more about how storage works in Aztec, read [the concepts](../../../../learn/concepts/storage/storage_slots.md).

[See the storage reference](../../references/storage/main.md).

:::info
The struct **must** be called `Storage` for the Aztec.nr library to properly handle it (this will be relaxed in the future).
:::

```rust
struct Storage {
  // public state variables
  // private state variables
}
```

If you have defined a `Storage` struct following this naming scheme, then it will be made available to you through the reserved `storage` keyword within your contract functions.

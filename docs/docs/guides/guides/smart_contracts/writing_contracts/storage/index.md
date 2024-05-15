---
title: Defining Storage
---

On this page, you will learn how to define storage in your smart contract.

To learn more about how storage works in Aztec, read [the concepts](/guides/guides/smart_contracts/writing_contracts/storage/storage_slots).

[See the storage reference](/aztec/aztec/concepts/storage/index.md).

```rust
#[aztec(storage)]
struct Storage {
  // public state variables
  // private state variables
}
```

If you have defined a struct and annotated it as `#[aztec(storage)]`, then it will be made available to you through the reserved `storage` keyword within your contract functions.

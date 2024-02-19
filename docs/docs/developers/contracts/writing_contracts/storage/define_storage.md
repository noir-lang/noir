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

Since Aztec.nr is written in Noir, which is state-less, we need to specify how the storage struct should be initialized to read and write data correctly. This is done by specifying an `init` function that is run in functions that rely on reading or altering the state variables. This `init` function must declare the Storage struct with an instantiation defining how variables are accessed and manipulated. The function MUST be called `init` for the Aztec.nr library to properly handle it (this will be relaxed in the future).

```rust
impl Storage {
  fn init(context: Context) -> Self {
    Storage {
      // (public state variables)::new()
      // (private state variables)::new()
    }
  }
}
```

If you have defined a `Storage` struct following this naming scheme, then it will be made available to you through the reserved `storage` keyword within your contract functions.

:::warning Using slot `0` is not supported!
No storage values should be initialized at slot `0` - storage slots begin at `1`. This is a known issue that will be fixed in the future.
:::

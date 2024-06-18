---
title: Global Variables
description: Documentation of Aztec's Global Variables in the Public and Private Contexts
sidebar_position: 2
---

# Global Variables

For developers coming from solidity, this concept will be similar to how the global `block` variable exposes a series of block values. The idea is the same in Aztec. Developers can access a namespace of values made available in each function.

`Aztec` has two execution environments, Private and Public. Each execution environment contains a different global variables object.

## Private Global Variables

#include_code tx-context /noir-projects/noir-protocol-circuits/crates/types/src/transaction/tx_context.nr rust

The private global variables are equal to the transaction context and contain:

### Chain Id

The chain id differs depending on which Aztec instance you are on ( NOT the Ethereum hardfork that the rollup is settling to ). On original deployment of the network, this value will be 1.

```rust
context.chain_id();
```

### Version

The version number indicates which Aztec hardfork you are on. The Genesis block of the network will have the version number 1.

```rust
context.version();
```

### Gas Settings

The gas limits set by the user for the transaction, the max fee per gas, and the inclusion fee.

## Public Global Variables

#include_code global-variables /noir-projects/noir-protocol-circuits/crates/types/src/abis/global_variables.nr rust

The public global variables contain the values present in the `private global variables` described above, with the addition of:

### Timestamp

The timestamp is the unix timestamp in which the block has been executed. The value is provided by the block's proposer (therefore can have variance). This value will always increase.

```rust
context.timestamp();
```

### Block Number

The block number is a sequential identifier that labels each individual block of the network. This value will be the block number of the block the accessing transaction is included in.
The block number of the genesis block will be 1, with the number increasing by 1 for every block after.

```rust
context.block_number();
```

:::info _Why do the available global variables differ per execution environment?_
The global variables are constrained by the proving environment. In the case of public functions, they are executed on a sequencer that will know the timestamp and number of the next block ( as they are the block producer ).
In the case of private functions, we cannot be sure which block our transaction will be included in, hence we can not guarantee values for the timestamp or block number.
:::

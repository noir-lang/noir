---
title: Published Gas & Fee Data
---

# Published Gas & Fee Data

When a block is published to L1, it includes information about the gas and fees at a block-level, and at a transaction-level.

## Block-level Data

The block header contains a `GlobalVariables`, which contains a `GasFees` object. This object contains the following fields:

- `feePerDaGas`: The fee in [Fee Juice](./fee-juice.md) per unit of DA gas consumed for transactions in the block.
- `feePerL2Gas`: The fee in FPA per unit of L2 gas consumed for transactions in the block.

`GlobalVariables` also includes a `coinbase` field, which is the L1 address that receives the fees.

The block header also contains a `totalFees` field, which is the total fees collected in the block in FPA.

## Updating the `GasFees` Object

Presently, the `feePerDaGas` and `feePerL2Gas` are fixed at `1` FPA per unit of DA gas and L2 gas consumed, respectively.

In the future, these values may be updated dynamically based on network conditions.

:::note Gas Targets
Should we move to a 1559-style fee market with block-level gas targets, there is an interesting point where gas "used" presently includes the entire [`teardown_gas_allocation`](./specifying-gas-fee-info.md) regardless of how much of that allocation was spent. In the future, if this becomes a concern, we can update our accounting to reflect the true gas used for the purposes of updating the `GasFees` object, though the user will be charged the full `teardown_gas_allocation` regardless.
:::

## Transaction-level Data

The transaction data which is published to L1 is a `TxEffects` object, which includes

- `transaction_fee`: the fee paid by the transaction in FPA

---
title: Fee Juice
---

# Fee Juice

Fee Juice is an enshrined asset in the Aztec network that is used to pay fees.

It has several important properties:

- It is fungible
- It cannot be transferred between accounts on the Aztec network
- It is obtained on Aztec via a bridge from Ethereum
- It only has public balances

All transactions on the Aztec network have a [non-zero transaction_fee](./fee-schedule.md#da-gas), denominated in FPA, which must be paid for the transaction to be included in the block.

When a block is successfully published on L1, the sequencer is paid on L1 the sum of all transaction fees in the block, denominated in FPA.

:::danger
We need a definition of the L1 fee juice.
:::

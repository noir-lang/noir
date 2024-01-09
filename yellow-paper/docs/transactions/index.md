---
title: Transactions
---

# Transactions

A transaction is the minimal action that changes the state of the network. Transactions in Aztec have a private and a public component, where the former is executed in the user's private execution environment (PXE) and the latter by the sequencer.

A transaction is also split into three phases to [support authorization abstraction and fee payments](../gas-and-fees/gas-and-fees.md#fees): a validation and fee preparation phase, a main execution phase, and fee distribution phase.

Users initiate a transaction by sending a _transaction request_ to their local PXE, which [locally simulates and proves the transaction](./local-execution.md) and returns a [_transaction_ object](./tx-object.md) identified by a [_transaction hash_](./tx-object.md#transaction-hash). This transaction object is then broadcasted to the network via an Aztec Node, which checks its [validity](./validity.md), and eventually picked up by a sequencer who [executes the public component of the transaction](./public-execution.md) and includes it in a block.

import DocCardList from '@theme/DocCardList';

<DocCardList />

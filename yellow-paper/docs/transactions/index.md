---
title: Transactions
---

# Transactions

A transaction is the minimal action that changes the state of the network. Transactions in Aztec have a private and a public component, where the former is executed in the user's private execution environment (PXE) and the latter by the sequencer.

A transaction is also split into three phases to [support authorization abstraction and fee payments](../gas-and-fees/index.md#fees): a validation and fee preparation phase, a main execution phase, and fee distribution phase.

Users initiate a transaction by sending a `transaction_request`<!-- link to defn. Is the `TransactionRequest` defined in circuits/private-kernel-initial the correct definition for this context? Or is it the "Execution Request" that's defined in the `./local-execution` section? --> to their local PXE, which [locally simulates and proves the transaction](./local-execution.md) and returns a [`transaction_object`](./tx-object.md#transaction-object-struct) identified by a [`transaction_hash`](./tx-object.md#transaction-hash). This transaction object is then broadcast to the network via an Aztec Node, which checks its [validity](./validity.md), and is eventually picked up by a sequencer who [executes the public component of the transaction](./public-execution.md) and includes it in a block.

import DocCardList from '@theme/DocCardList';

<DocCardList />

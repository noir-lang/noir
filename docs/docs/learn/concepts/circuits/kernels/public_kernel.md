---
title: Public Kernel Circuit
---

This circuit is executed by a Sequencer, since only a Sequencer knows the current state of the [public data tree](../../storage/trees/main.md#public-state-tree) at any time. A Sequencer might choose to delegate proof generation to the Prover pool.

- Exposes (forwards) the following data to the next recursive circuit:
  - all data accumulated by all previous private kernel circuit recursions of this tx;
  - all data accumulated by all previous public kernel circuit recursions of this tx;
  - new public state read requests;
  - new public state transition requests;
  - new messages to L1 contracts;
  - new messages to private L2 functions;
  - public call stacks: hashes representing calls to other public functions;
  - events;
- Verifies a previous 'Private/Public Kernel Proof', recursively, when verifying transactions which are composed of many function calls.
- Ensures the entire stack trace of public functions (for a particular tx) adheres to function execution rules.

---
title: Circuits
---

## Motivation

In Aztec, circuits come from two sources:

1. Core protocol circuits
2. User-written circuits (written as Noir Contracts and deployed to the network)

This page focusses on the core protocol circuits. These circuits check that the rules of the protocol are being adhered to.

When a function in an Ethereum smart contract is executed, the EVM performs checks to ensure that Ethereum's transaction rules are being adhered-to correctly. Stuff like:

- "Does this tx have a valid signature?"
- "Does this contract address contain deployed code?
- "Does this function exist in the requested contract?"
- "Is this function allowed to call this function?"
- "How much gas has been paid, and how much is left?"
- "Is this contract allowed to read/update this state variable?"
- "Perform the state read / state write"
- "Execute these opcodes"

All of these checks have a computational cost, for which users are charged gas.

Many existing L2s move this logic off-chain, as a way of saving their users gas costs, and as a way of increasing tx throughput.

zk-Rollups, in particular, move these checks off-chain by encoding them in zk-S(N/T)ARK circuits. Rather than paying a committee of Ethereum validators to perform the above kinds of checks, L2 users instead pay a sequencer to execute these checks via the circuit(s) which encode them. The sequencer can then generate a zero-knowledge proof of having executed the circuit(s) correctly, which they can send to a rollup contract on Ethereum. The Ethereum validators then verify this zk-S(N/T)ARK. It often turns out to be much cheaper for users to pay the sequencer to do this, than to execute a smart contract on Ethereum directly.

But there's a problem.

Ethereum (and the EVM) doesn't have a notion of privacy.

- There is no notion of a private state variable in the EVM.
- There is no notion of a private function in the EVM.

So users cannot keep private state variables' values private from Ethereum validators, nor from existing (non-private) L2 sequencers. Nor can users keep the details of which function they've executed private from validators or sequencers.

How does Aztec add privacy?

Well, we just encode _extra_ checks in our zk-Rollup's zk-SNARK circuits! These extra checks introduce the notions of private state and private functions, and enforce privacy-preserving constraints on every transaction being sent to the network.

In other words, since neither the EVM nor other rollups have rules for how to preserve privacy, we've written a new rollup which introduces such rules, and we've written circuits to enforce those rules!

What kind of extra rules / checks does a rollup need, to enforce notions of private states and private functions? Stuff like:

- "Perform state reads and writes using new tree structures which prevent tx linkability" (see [trees](trees)).
- "Hide which function was just executed, by wrapping it in a zk-snark"
- "Hide all functions which were executed as part of this tx's stack trace, by wrapping the whole tx in a zk-snark"

## Aztec core protocol circuits

So what kinds of core protocol circuits does Aztec have?

### Kernel Circuits

- [Private Kernel Circuit](./circuits/private-kernel)
- [Public Kernel Circuit](./circuits/public-kernel)

### Rollup Circuits

The primary purpose of the Rollup Circuits is to 'squish' all of the many thousands of transactions in a rollup into a single SNARK, which can then be efficiently (cheaply) verified on Ethereum.

These circuits are executed by the Sequencer, since their primary role is to order transactions. The Sequencer might choose to delegate proof generation to the Prover pool.

The way we 'squish' all this data is in a 'binary tree of proofs' topology.

> Example: If there were 16 txs in a rollup, we'd arrange the 16 kernel proofs into 8 pairs and merge each pair into a single proof (using zk-snark recursion techniques), resulting in 8 output proofs. We'd then arrange those 8 proofs into pairs and again merge each pair into a single proof, resulting in 4 output proofs. And so on until we'd be left with a single proof, which represents the correctness of the original 16 txs.

This 'binary tree of proofs' topology allows proof generation to be greatly parallelised across prover instances. Each layer of the tree can be computed in parallel. Or alternatively, subtrees can be coordinated to be computed in parallel.

> Note: 'binary tree of proofs' is actually an oversimplification. The Rollup Circuits are designed so that a Sequencer can actually deviate from a neat, symmetrical tree, for the purposes of efficiency, and instead sometimes create wonky trees.

> Some of the Rollup Circuits also do some protocol checks and computations, for efficiency reasons. We might rearrange which circuit does what computation, as we discover opportunities for efficiency.

#### Base Rollup Circuit

The Base Rollup Circuit takes two kernel circuit proofs (from two distinct transactions) and compresses them into a single snark.

For both transactions, it:

- Performs public state read membership checks.
- Updates the public data tree in line with the requested state transitions.
- Checks that the nullifiers haven't previously been inserted into the [indexed nullifier tree](./trees/indexed-merkle-tree).
- Batch-inserts new nullifiers into the nullifier tree.
- Batch-inserts new commitments into the private data tree
- Batch-inserts any new contract deployments into the contract tree.
- Hashes all the new nullifiers, commitments, public state transitions, and new contract deployments, to prevent exponential growth in public inputs with each later layer of recursion.
- Verifies the input kernel proof.

#### Merge Rollup Circuit

The Merge Rollup Circuit takes two Base or Merge circuit proofs and compresses them into a single snark.

It:

- Hashes the two input hashes into a single output hash.
- Verifies the two input base or merge proofs.

#### Root Rollup Circuit

The Root Rollup Circuit takes two Base or Merge circuit proofs and compresses them into a single snark.

It:

- Hashes the two input hashes into a single output hash.
- Verifies the two input base or merge proofs.

#### Squisher Circuits

We haven't fully spec'ed these out, as Honk and Goblin Plonk schemes are still being improved! But we'll need some extra circuit(s) to squish a Honk proof (as produced by the Root Rollup Circuit) into a Standard Plonk or Fflonk proof, for cheap verification on Ethereum.

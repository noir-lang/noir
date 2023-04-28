---
title: What is Aztec?
---

Aztec is a Layer 2 zk-rollup, designed to enable programmable privacy, that will run on Ethereum. It will allow developers to build fully programable, privacy preserving smart contracts, with composable call semantics using our ZK-Snark programming language, [Noir](./noir).

**Why?**

A public blockchain utilizes a peer-to-peer network and a consensus protocol to establish the correct record of events. The core unit of intent (a transaction) is a request to update state, based on the logic of a predefined program. The blockchain node, computes this state update and records it on a shared ledger. The correctness of the ledger is enforced by other nodes "checking" the work of the current node -- only possible as the transactions and their data are public and visible to anyone.

Ethereum is an example of a public blockchain, that enables the processing of transactions with arbitrary, Turing complete computation.

**Aztec is an encrypted blockchain**, where the core unit of intent is a zero-knowledge proof, not a transaction request. The zero-knowledge proof, proves the correct execution of a specific transaction and any resultant state updates.

Individual transaction proofs are recursively aggregated or "rolled up" using a zk-rollup construction, for final verification on Ethereum.

To read more about the network, how it works, and the types of applications that can be built, head to Aztec's Architecture [here](./aztec/overview).

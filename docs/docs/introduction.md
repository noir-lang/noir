---
id: intro
title: Introduction
slug: "/"
---
import Disclaimer from './common/_disclaimer.mdx';

<Disclaimer/>

## Aztec's Vision

Our vision is to create a fairer, more open financial eco-system, built with encryption at its core.

We believe decentralization is premised on individual rights -- without widely accessible encryption, we compromise our ability to choose how we live our lives and earn our livelihoods.

To acheive this goal, we are building the [Aztec Network](https://aztec.network/), a fully programmable private [ZK-rollup](https://ethereum.org/en/developers/docs/scaling/zk-rollups/) on [Ethereum](https://ethereum.org/). Enabling developers to create decentralized applications with encryption and scale.

**Network Values**

- **Private.** -- The only zero-knowledge rollup built with a privacy-first UTXO architecture to allow developers to build privacy preserving programable applications on the Aztec rollup.
- **Accessible.** -- Proving transaction validity via recursively validation transactions through zero-knowledge proofs on Ethereum significantly reduces transaction costs.
- **Compliant.**  -- The programmable nature of Aztec smart contracts, enables dApp developers to code privacy-preserving auditability and compliance while fully preserving a credible neutral protocol layer.

> “When we started Aztec, the technology to scale blockchains privately didn’t exist. Since then, we’ve assembled a team of world-class cryptographers who continuously redefine the state-of-the-art. Inventing PLONK — the paradigm-defining universal zk-SNARK — showcases our ability to produce technology that matches our ambitions: unlocking an entire universe of blockchain applications that couldn’t exist without privacy.” _- Zac Williamson, CEO and Cofounder, Aztec_

We are pioneering the cryptography and reserach to bring our next generation, privacy-preserving zk-roll-up to mainnet, codenamed Aztec 3.

## What is Aztec?

Aztec is an encrytped blockchain, built as a Layer 2 running on Ethereum. It allows developers to build fully programable, privacy preserving smart contracts, with composable call semantics using our ZK-Snark programming language Noir.

**Why?**

First, some context. A public blockchain utilizes a peer-to-peer network and a consensus protocol to establish the correct record of events. The core unit of work (a transaction) is a request to update state, based on the logic of a predefined program. The blockchain node, computes this state update and records it on a shared ledger. The correctness of the ledger is enforced by other nodes "checking" the work of the current node -- only possible as the transactions and their data are public and visible to anyone.

Ethereum is an example of a public blockchain, that enables the processing of transactions with arbitrary, Turing complete computation.

**Aztec is an encrypted blockchain**, where the core unit of work is a zero-knowledge proof, not a transaction request. The zero-knowledge proof, proves the correct execution of a specific transaction and any resultant state updates.

Individual transaciton proofs are recursively aggregated or "rolled up" using a zk-rollup construction, for final verification on Ethereum.

To read more about the network, how it works, and the types of applications that can be built, head to Aztec's Architecture [here](./aztec/overview).

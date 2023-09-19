---
title: State Model
---

import Disclaimer from '../../misc/common/\_disclaimer.mdx';

<Disclaimer/>

## Private State

Private state must be treated differently from public state and this must be expressed in the semantics of Aztec.nr.

Private state is encrypted and therefore is "owned" by a user or a set of users (via shared secrets) that are able to decrypt the state.

Private state is represented in an append-only database since updating a record would leak information about the transaction graph.

The act of "deleting" a private state variable can be represented by adding an associated nullifier to a nullifier set. The nullifier is generated such that, without knowing the decryption key of the owner, an observer cannot link a state record with a nullifier.

Modification of state variables can be emulated by nullifying the a state record and creating a new record to represent the variable. Private state has an intrinsic UTXO structure and this must be represented in the language semantics of manipulating private state.

### Abstracting UTXO's from App's / Users

The goal of the Aztec.nr smart contract library is to abstract the UTXO model away from an app user / developer, contract developers are the only actor who should have to think about UTXO's.

This is achieved with two main features:

1. Users sign over transactions, not over specific UTXO's
2. Aztec.nr contracts support developer defined `unconstrained` getter functions to help dApp's make sense of UTXO's. e.g `getBalance()`. These functions can be called outside of a transaction context to read private state.

## Coming soon

- Public State
- The lifecycle of a note
- Custom notes
- Injection of data by the kernel
- Nonce & contract address
- Custom nullifiers
- Emission of custom note data to L1
- Decrypting and storing encrypted note data

---
title: Sandbox Reference
sidebar_position: 0
---

The Aztec Sandbox is an environment for local development on the Aztec Network. It's easy to get setup with just a single, simple command, and contains all the components needed to develop and test Aztec contracts and applications.

## What's in the Sandbox?

The sandbox contains a local Ethereum instance running [Anvil](https://book.getfoundry.sh/anvil/), a local instance of the Aztec rollup and an aztec private execution client for handling user transactions and state.

These provide a self contained environment which deploys Aztec on a local (empty) Ethereum network, creates 3 smart contract wallet accounts on the rollup, and allows transactions to be processed on the local Aztec sequencer.

The current sandbox does not generate or verify proofs, but provides a working end to end developer flow for writing and interacting with Aztec.nr smart contracts.

## Command line tools

Aztec-nargo and aztec CLI are command-line tool allowing you to compile smart contracts. See the [compiling contracts](../../guides/smart_contracts/how_to_compile_contract.md) page for more information.
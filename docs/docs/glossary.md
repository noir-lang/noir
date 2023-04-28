---
title: Glossary
---

### Barretenberg

Aztec's cryptography back-end. Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Sequencer

This service is responsible for:

- Watching for rollup blocks on Ethereum and updating the representation of Aztec state accordingly
- Listening for and storing transactions from users, verifying they're valid, have correct fees, etc.
- Constructing new rollups at the appropriate time or when enough transactions are received
- Publishing of rollups to an Ethereum chain

You can find the Typescript reference implementation called Falafel [here](https://github.com/AztecProtocol/aztec-connect/tree/master/falafel).

Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Smart Contract

Programs that run on the Aztec network are called smart contracts, similar to [programs](https://ethereum.org/en/developers/docs/smart-contracts/) that run on Ethereum.

Smart contracts on Aztec may also optionally include private state and private functions.

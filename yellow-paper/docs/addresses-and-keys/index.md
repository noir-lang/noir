---
title: Addresses and Keys
---

Aztec has no concept of externally-owned accounts. Every address is meant to identify a smart contract in the network. Addresses are then a commitment to a contract class, a list of constructor arguments, and a set of keys.

Keys in Aztec are used both for authorization and privacy. Authorization keys are managed by account contracts, and not mandated by the protocol. Each account contract may use different authorization keys, if at all, with different signing mechanisms.

Privacy keys are used for note encryption, tagging, and nullifying. These are also not enforced by the protocol. However, for facilitating composability, the protocol enshrines a set of well-known encryption and tagging mechanisms, that can be leveraged by applications as they interact with accounts.

The [specification](./specification.md) covers the main requirements for addresses and keys, along with their specification and derivation mechanisms, while the [precompiles](./precompiles.md) section describes well-known contract addresses, with implementations defined by the protocol, used for note encryption and tagging.

Last, the [diversified and stealth accounts](./diversified-and-stealth.md) sections describe application-level recommendations for diversified and stealth accounts.

import DocCardList from '@theme/DocCardList';

<DocCardList />

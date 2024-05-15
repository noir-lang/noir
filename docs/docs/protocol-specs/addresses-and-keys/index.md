---
title: Addresses and Keys
---
import DocCardList from '@theme/DocCardList';

Aztec has no concept of externally-owned accounts. Every address identifies a smart contract in the network.

For end users to interact with the network, they'll most-likely need to [deploy](../contract-deployment/index.md) a so-called "account contract".

Addresses are then a commitment to a contract class, a list of constructor arguments, and a set of keys.

Keys in Aztec are used both for authorization and privacy. Authorization keys are managed by account contracts, and not mandated by the protocol. Each account contract may use different authorization keys, if at all, with different signing mechanisms.

Privacy keys are used for note encryption, tagging, and nullifying. These are also not enforced by the protocol. However, for facilitating composability, the protocol enshrines a set of enshrined encryption and tagging mechanisms, that can be leveraged by applications as they interact with accounts.

The [requirements](./keys-requirements.md) section outlines the features that were sought when designing Aztec's addresses and keys. We then specify how [addresses](./address.md) are derived, as well as the default way in which [keys](./keys.md) will be derived. The [precompiles](./precompiles.md) section describes enshrined contract addresses, with implementations defined by the protocol, used for note encryption and tagging.

Last, the [diversified and stealth accounts](./diversified-and-stealth.md) sections describe application-level recommendations for diversified and stealth accounts.


<DocCardList />

---
title: Oracle Functions
---

This page goes over what oracles are in Aztec and how they work.

Looking for a hands-on guide? You can learn how to use oracles in a smart contract [here](../../../../guides/smart_contracts/writing_contracts/how_to_pop_capsules.md).

An oracle is something that allows us to get data from the outside world into our contracts. The most widely-known types of oracles in blockchain systems are probably Chainlink price feeds, which allow us to get the price of an asset in USD taking non-blockchain data into account.

While this is one type of oracle, the more general oracle, allows us to get any data into the contract. In the context of oracle functions or oracle calls in Aztec, it can essentially be seen as user-provided arguments, that can be fetched at any point in the circuit, and don't need to be an input parameter.

**Why is this useful? Why don't just pass them as input parameters?**
In the world of EVM, you would just read the values directly from storage and call it a day. However, when we are working with circuits for private execution, this becomes more tricky as you cannot just read the storage directly from your state tree, because there are only commitments (e.g. hashes) there. The pre-images (content) of your commitments need to be provided to the function to prove that you actually allowed to modify them.

If we fetch the notes using an oracle call, we can keep the function signature independent of the underlying data and make it easier to use. A similar idea, applied to the authentication mechanism is used for the Authentication Witnesses that allow us to have a single function signature for any wallet implementation, see [AuthWit](../../wallets/index.md#authorizing-actions) for more information on this.

Oracles introduce **non-determinism** into a circuit, and thus are `unconstrained`. It is important that any information that is injected into a circuit through an oracle is later constrained for correctness. Otherwise, the circuit will be **under-constrained** and potentially insecure!

`Aztec.nr` has a module dedicated to its oracles. If you are interested, you can view them by following the link below:
#include_code oracles-module /noir-projects/aztec-nr/aztec/src/oracle/mod.nr rust

## Inbuilt oracles

- [`debug_log`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/aztec/src/oracle/debug_log.nr) - Provides a couple of debug functions that can be used to log information to the console. Read more about debugging [here](../../../../reference/debugging.md#logging-in-aztecnr).
- [`auth_witness`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/authwit/src/auth_witness.nr) - Provides a way to fetch the authentication witness for a given address. This is useful when building account contracts to support approve-like functionality.
- [`get_l1_to_l2_message`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/aztec/src/oracle/get_l1_to_l2_message.nr) - Useful for application that receive messages from L1 to be consumed on L2, such as token bridges or other cross-chain applications.
- [`notes`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/aztec/src/oracle/notes.nr) - Provides a lot of functions related to notes, such as fetches notes from storage etc, used behind the scenes for value notes and other pre-build note implementations.
- [`logs`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/aztec/src/oracle/logs.nr) - Provides the to log encrypted and unencrypted data.

Find a full list [on GitHub](https://github.com/AztecProtocol/aztec-packages/tree/master/noir-projects/aztec-nr/aztec/src/oracle).

Please note that it is **not** possible to write a custom oracle for your dapp. Oracles are implemented in the PXE, so all users of your dapp would have to use a PXE service with your custom oracle included. If you want to inject some arbitrary data that does not have a dedicated oracle, you can use [popCapsule](../../../../guides/smart_contracts/writing_contracts/how_to_pop_capsules.md).

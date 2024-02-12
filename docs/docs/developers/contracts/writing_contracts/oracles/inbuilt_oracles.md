---
title: Inbuilt Oracles
---

This page goes over all the oracles that are available in Aztec.nr. If you'd like to read more about what oracles are, check out [this page](../oracles/main.md).

## Inbuilt oracles

- [`debug_log`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/aztec/src/oracle/debug_log.nr) - Provides a couple of debug functions that can be used to log information to the console. Read more about debugging [here](../../../debugging/main.md#logging-in-aztecnr).
- [`auth_witness`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/authwit/src/auth_witness.nr) - Provides a way to fetch the authentication witness for a given address. This is useful when building account contracts to support approve-like functionality.
- [`get_l1_to_l2_message`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/aztec/src/oracle/get_l1_to_l2_message.nr) - Useful for application that receive messages from L1 to be consumed on L2, such as token bridges or other cross-chain applications.
- [`notes`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/aztec/src/oracle/notes.nr) - Provides a lot of functions related to notes, such as fetches notes from storage etc, used behind the scenes for value notes and other pre-build note implementations.
- [`logs`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/aztec-nr/aztec/src/oracle/logs.nr) - Provides the to log encrypted and unencrypted data.

Find a full list [on GitHub](https://github.com/AztecProtocol/aztec-packages/tree/master/noir-projects/aztec-nr/aztec/src/oracle).

Please note that it is **not** possible to write a custom oracle for your dapp. Oracles are implemented in the PXE, so all users of your dapp would have to use a PXE service with your custom oracle included. If you want to inject some arbitrary data that does not have a dedicated oracle, you can use [popCapsule](./pop_capsule.md).

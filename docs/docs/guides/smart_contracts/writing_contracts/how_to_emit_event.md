---
title: Emitting Events
sidebar_position: 3
---

Events in Aztec work similarly to Ethereum events in the sense that they are a way for contracts to communicate with the outside world.
They are emitted by contracts and stored inside each instance of an AztecNode.

:::info
Aztec events are currently represented as raw data and are not ABI encoded.
ABI encoded events are a feature that will be added in the future.
:::

Unlike on Ethereum, there are 2 types of events supported by Aztec: [encrypted](#encrypted-events) and [unencrypted](#unencrypted-events).

## Encrypted Events

### Register a recipient

Encrypted events can only be emitted by private functions and are encrypted using a public key of a recipient.
For this reason it is necessary to register a recipient in the Private Execution Environment (PXE) before encrypting the events for them.

First we need to get a hold of recipient's [complete address](../../../aztec/concepts/accounts/keys#complete-address).
Below are some ways how we could instantiate it after getting the information in a string form from a recipient:

#include_code instantiate-complete-address /yarn-project/circuits.js/src/structs/complete_address.test.ts rust

Then to register the recipient's complete address in PXE we would call `registerRecipient` PXE endpoint using [Aztec.js](../../../aztec/core_components.md#aztecjs):

#include_code register-recipient /yarn-project/aztec.js/src/wallet/create_recipient.ts rust

:::info
If a note recipient is one of the accounts inside the PXE, we don't need to register it as a recipient because we already have the public key available. You can register a recipient as shown [here](../how_to_deploy_contract.md)
:::

### Call emit

To emit encrypted logs you can import the `encode_and_encrypt` or `encode_and_encrypt_with_keys` functions and pass them into the `emit` function after inserting a note. An example can be seen in the reference token contract's transfer function:

#include_code encrypted /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

Furthermore, if not emitting the note, one should explicitly `discard` the value returned from the note creation.

### Successfully process the encrypted event

One of the functions of the PXE is constantly loading encrypted logs from the `AztecNode` and decrypting them.
When new encrypted logs are obtained, the PXE will try to decrypt them using the private encryption key of all the accounts registered inside PXE.
If the decryption is successful, the PXE will store the decrypted note inside a database.
If the decryption fails, the specific log will be discarded.

For the PXE to successfully process the decrypted note we need to compute the note's 'note hash' and 'nullifier'.
Aztec.nr enables smart contract developers to design custom notes, meaning developers can also customize how a note's note hash and nullifier should be computed. Because of this customizability, and because there will be a potentially-unlimited number of smart contracts deployed to Aztec, an PXE needs to be 'taught' how to compute the custom note hashes and nullifiers for a particular contract. This is done by a function called `compute_note_hash_and_optionally_a_nullifier`, which is automatically injected into every contract when compiled.

## Encrypted Events

To emit generic event information as an encrypted log, call the context method `encrypt_and_emit_note`. Currently, only arrays of
fields are supported, and the PXE will fail to decrypt, process and store this data, so it will not be queryable automatically.

## Unencrypted Events

Unencrypted events are events which can be read by anyone.
They can be emitted by both public and private functions.

:::danger

- Emitting unencrypted events from private function is a significant privacy leak and it should be considered by the developer whether it is acceptable.

:::

### Call emit_unencrypted_log

To emit unencrypted logs you don't need to import any library. You call the context method `emit_unencrypted_log`:

#include_code emit_unencrypted /noir-projects/noir-contracts/contracts/test_contract/src/main.nr rust

### Querying the unencrypted event

Once emitted, unencrypted events are stored in AztecNode and can be queried by anyone:

#include_code get_logs /yarn-project/end-to-end/src/fixtures/utils.ts typescript

## Costs

All event data is pushed to Ethereum as calldata by the sequencer and for this reason the cost of emitting an event is non-trivial.

In the Sandbox, an encrypted note has a fixed overhead of 4 field elements (to broadcast an ephemeral public key, a contract address, and a storage slot); plus a variable number of field elements depending on the type of note being emitted.

A `ValueNote`, for example, currently uses 3 fields elements (plus the fixed overhead of 4). That's roughly `7 * 32 = 224` bytes of information.

#include_code value-note-def /noir-projects/aztec-nr/value-note/src/value_note.nr

- There are plans to compress encrypted note data further.
- There are plans to adopt EIP-4844 blobs to reduce the cost of data submission further.

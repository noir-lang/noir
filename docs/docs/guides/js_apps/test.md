---
title: Testing Aztec.nr contracts with TypeScript
---

In this guide we will cover how to interact with your Aztec.nr smart contracts in a testing environment to write automated tests for your apps.

## Prerequisites

- A compiled contract with TS interface (read [how to compile](../smart_contracts/how_to_compile_contract.md))
- Your sandbox running (read [getting started](../../getting_started.md))

## Create TS file and install libraries

Pick where you'd like your tests to live and create a Typescript project.

You will need to install Aztec.js:

```bash
yarn add @aztec/aztecjs
```

You can use `aztec.js` to write assertions about transaction statuses, about chain state both public and private, and about logs.

## Import relevant libraries

Import `aztecjs`. This is an example of some functions and types you might need in your test:

#include_code imports /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

You should also import the [Typescript class you generated](../smart_contracts/how_to_compile_contract.md#typescript-interfaces):

#include_code import_contract /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

## Create a PXE client

Currently, testing Aztec.nr smart contracts means testing them against the PXE that runs in the local sandbox. Create a PXE client:

#include_code create_pxe_client /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

and use the accounts that are initialized with it:

#include_code use-existing-wallets /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

Alternatively, you can [create a new account.](./create_account.md).

## Write tests

### Calling and sending transactions

You can send transactions within your tests with Aztec.js. Read how to do that in these guides:

- [Call a view (unconstrained) function](./call_view_function.md)
- [Send a transaction](./send_transaction.md)

### Using debug options

You can use the `debug` option in the `wait` method to get more information about the effects of the transaction. This includes information about new note hashes added to the note hash tree, new nullifiers, public data writes, new L2 to L1 messages, new contract information, and newly visible notes.

This debug information will be populated in the transaction receipt. You can log it to the console or use it to make assertions about the transaction.

#include_code debug /yarn-project/end-to-end/src/e2e_token_contract/minting.test.ts typescript

You can also log directly from Aztec contracts. Read [this guide](/reference/debugging.md##logging-in-aztecnr) for some more information.

### Examples

#### A private call fails

We can check that a call to a private function would fail by simulating it locally and expecting a rejection. Remember that all private function calls are only executed locally in order to preserve privacy. As an example, we can try transferring more tokens than we have, which will fail an assertion with the `Balance too low` error message.

#include_code local-tx-fails /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

Under the hood, the `send()` method executes a simulation, so we can just call the usual `send().wait()` to catch the same failure.

#include_code local-tx-fails-send /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### A transaction is dropped

We can have private transactions that work fine locally, but are dropped by the sequencer when tried to be included due to a double-spend. In this example, we simulate two different transfers that would succeed individually, but not when both are tried to be mined. Here we need to `send()` the transaction and `wait()` for it to be mined.

#include_code tx-dropped /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### A public call fails locally

Public function calls can be caught failing locally similar to how we catch private function calls. For this example, we use a [`TokenContract`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/noir-contracts/contracts/token_contract/src/main.nr) instead of a private one.

#include_code local-pub-fails /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### A public call fails on the sequencer

We can ignore a local simulation error for a public function via the `skipPublicSimulation`. This will submit a failing call to the sequencer, who will include the transaction, but without any side effects from our application logic. Requesting the receipt for the transaction will also show it has a reverted status.

#include_code pub-reverted /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

```
WARN Error processing tx 06dc87c4d64462916ea58426ffcfaf20017880b353c9ec3e0f0ee5fab3ea923f: Assertion failed: Balance too low.
```

### Querying state

We can check private or public state directly rather than going through view-only methods, as we did in the initial example by calling `token.methods.balance().simulate()`.

To query storage directly, you'll need to know the slot you want to access. This can be checked in the [contract's `Storage` definition](../../reference/smart_contract_reference/storage/index.md) directly for most data types. However, when it comes to mapping types, as in most EVM languages, we'll need to calculate the slot for a given key. To do this, we'll use the [`CheatCodes`](../../reference/sandbox_reference/cheat_codes.md) utility class:

#include_code calc-slot /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### Querying private state

Private state in the Aztec is represented via sets of [private notes](../../aztec/concepts/state_model/index.md#private-state). We can query the Private Execution Environment (PXE) for all notes encrypted for a given user in a contract slot. For example, this gets all notes encrypted for the `owner` user that are stored on the token contract address and on the slot that was calculated earlier. To calculate the actual balance, it extracts the `value` of each note, which is the first element, and sums them up.

#include_code private-storage /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### Querying public state

[Public state](../../aztec/concepts/state_model/index.md#public-state) behaves as a key-value store, much like in the EVM. We can directly query the target slot and get the result back as a buffer. Note that we use the [`TokenContract`](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/noir-contracts/contracts/token_contract/src/main.nr) in this example, which defines a mapping of public balances on slot 6.

#include_code public-storage /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

### Logs

You can check the logs of [events](../smart_contracts/writing_contracts/how_to_emit_event.md) emitted by contracts. Contracts in Aztec can emit both [encrypted](../smart_contracts/writing_contracts/how_to_emit_event.md#Encrypted-Events) and [unencrypted](../smart_contracts/writing_contracts/how_to_emit_event.md#Unencrypted-Events) events.

#### Querying unencrypted logs

We can query the PXE for the unencrypted logs emitted in the block where our transaction is mined. Logs need to be unrolled and formatted as strings for consumption.

#include_code unencrypted-logs /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

## Cheats

The [`CheatCodes`](../../reference/sandbox_reference/cheat_codes.md) class, which we used for [calculating the storage slot above](#querying-state), also includes a set of cheat methods for modifying the chain state that can be handy for testing.

### Set next block timestamp

The `warp` method sets the time for next execution, both on L1 and L2. We can test this using an `isTimeEqual` function in a `Test` contract defined like the following:

#include_code is-time-equal noir-projects/noir-contracts/contracts/test_contract/src/main.nr rust

We can then call `warp` and rely on the `isTimeEqual` function to check that the timestamp was properly modified.

#include_code warp /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

## Further reading

- [How to call a view transactions in Aztec.js](./call_view_function.md)
- [How to send a transactions in Aztec.js](./send_transaction.md)
- [How to deploy a contract in Aztec.js](./deploy_contract.md)
- [How to create an account in Aztec.js](./create_account.md)
- [Cheat codes](../../reference/sandbox_reference/cheat_codes.md)
- [How to compile a contract](../smart_contracts/how_to_compile_contract.md).

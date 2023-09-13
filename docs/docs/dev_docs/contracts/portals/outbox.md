---
title: Outbox
---

The `Outbox` is a contract deployed on L1 that handles message passing from the rollup and to L1.

**Links**: [Interface](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol), [Implementation](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/messagebridge/Outbox.sol).

## `sendL1Messages()`

Inserts multiple messages from the `Rollup`.

#include_code outbox_send_l1_msg l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol solidity

| Name           | Type    | Description |
| -------------- | ------- | ----------- |
| `_entryKeys`         | `bytes32[]` | A list of message hashes to insert into the outbox for later consumption |

#### Edge cases

- Will revert with `Registry__RollupNotRegistered(address rollup)` if `msg.sender` is not registered as a rollup on the [`Registry`](./registry.md)
- Will revert `Outbox__IncompatibleEntryArguments(bytes32 entryKey, uint64 storedFee, uint64 feePassed, uint32 storedVersion, uint32 versionPassed, uint32 storedDeadline, uint32 deadlinePassed)` if insertion is not possible due to invalid entry arguments.

## `consume()`

Allows a recipient to consume a message from the `Outbox`.

#include_code outbox_consume l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol solidity


| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_message`     | `L2ToL1Msg` | The message to consume |
| ReturnValue    | `bytes32`   | The hash of the message | 

#### Edge cases

- Will revert with `Outbox__Unauthorized()` if `msg.sender != _message.recipient.actor`. 
- Will revert with `Outbox__InvalidChainId()` if `block.chainid != _message.recipient.chainId`.
- Will revert with `Outbox__NothingToConsume(bytes32 entryKey)` if the message does not exist.
- Will revert with `Outbox__InvalidVersion(uint256 entry, uint256 message)` if the version of the entry and message sender don't match (wrong rollup).

## `get()`
Retrieves the `entry` for a given message. The entry contains fee, occurrences, deadline and version information. 

#include_code outbox_get l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol solidity

| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_entryKey`    | `bytes32`   | The entry key (message hash) |
| ReturnValue    | `Entry`     | The entry for the given key | 

#### Edge cases
- Will revert with `Outbox__NothingToConsume(bytes32 entryKey)` if the message does not exist.

## `contains()`
Returns whether the key is found in the inbox.

#include_code outbox_contains l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol solidity

| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_entryKey`    | `bytes32`   | The entry key (message hash)|
| ReturnValue    | `bool`   | True if contained, false otherwise| 

## `computeEntryKey()`
Computes the hash of a message.

#include_code outbox_compute_entry_key l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol solidity

| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_message`     | `L2ToL1Msg` | The message to compute hash for |
| ReturnValue    | `bytes32`   | The hash of the message | 
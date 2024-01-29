---
title: Inbox
---

The `Inbox` is a contract deployed on L1 that handles message passing from L1 to the rollup (L2)

**Links**: [Interface](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/interfaces/messagebridge/IInbox.sol), [Implementation](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/messagebridge/Inbox.sol).

## `sendL2Message()`

Sends a message from L1 to L2.

#include_code send_l1_to_l2_message l1-contracts/src/core/interfaces/messagebridge/IInbox.sol solidity


| Name           | Type    | Description |
| -------------- | ------- | ----------- |
| Recipient      | `L2Actor` | The recipient of the message. This **MUST** match the rollup version and an Aztec contract that is **attached** to the contract making this call. If the recipient is not attached to the caller, the message cannot be consumed by it. |
| Deadline       | `uint256` | The message consumption deadline. If the message have not been removed from the `Inbox` and included in a rollup block by this point, it can be *canceled* by the portal (the portal must implement logic to cancel). |
| Content        | `field` (~254 bits) | The content of the message. This is the data that will be passed to the recipient. The content is limited to be a single field for rollup purposes. If the content is small enough it can just be passed along, otherwise it should be hashed and the hash passed along (you can use our [`Hash`](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/libraries/Hash.sol) utilities with `sha256ToField` functions)  |
| Secret Hash    | `field` (~254 bits)  | A hash of a secret that is used when consuming the message on L2. Keep this preimage a secret to make the consumption private. To consume the message the caller must know the pre-image (the value that was hashed) - so make sure your app keeps track of the pre-images! Use the [`computeMessageSecretHash`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec.js/src/utils/secrets.ts) to compute it from a secret. |
| Fee (msg.value)     | `uint256`  | The fee to the sequencer for including the message. This is the amount of ETH that the sequencer will receive for including the message. Note that only values that can fit in `uint64` will be accepted |
| ReturnValue         | `bytes32` | The message hash, used as an identifier |

#### Edge cases

- Will revert with `Inbox__ActorTooLarge(bytes32 actor)` if the recipient is larger than the field size (~254 bits).
- Will revert with `Inbox__DeadlineBeforeNow()` if the deadline is before the current block.
- Will revert with `Inbox__ContentTooLarge(bytes32 content)` if the content is larger than the field size (~254 bits).
- Will revert with `Inbox__SecretHashTooLarge(bytes32 secretHash)` if the secret hash is larger than the field size (~254 bits).
- Will revert with `Inbox__FeeTooHigh()` if the fee is larger than `type(uint64).max`.
- Will revert `Inbox__IncompatibleEntryArguments(bytes32 entryKey, uint64 storedFee, uint64 feePassed, uint32 storedVersion, uint32 versionPassed, uint32 storedDeadline, uint32 deadlinePassed)` if insertion is not possible due to invalid entry arguments.

## `cancelL2Message()`
Cancels a message that has not yet been consumed.

#include_code pending_l2_cancel l1-contracts/src/core/interfaces/messagebridge/IInbox.sol solidity

| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_message`     | `L1ToL2Msg` | The message to cancel |
| `_feeCollector`| `address`   | The address to refund the fee to |
| ReturnValue    | `bytes32`   | The hash of the message | 

#### Edge cases

- Will revert with `Inbox__Unauthorized()` if `msg.sender != _message.sender.actor`. 
- Will revert with `Inbox__NotPastDeadline()` if `block.timestamp <= _message.deadline`.
- Will revert with `Inbox__NothingToConsume(bytes32 entryKey)` if the message does not exist.

## `batchConsume()`

Allows the `Rollup` to consume multiple messages in a single transaction.

#include_code inbox_batch_consume l1-contracts/src/core/interfaces/messagebridge/IInbox.sol solidity

| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_entryKeys`   | `bytes32[]` | The entry keys (message hashes) to consume |
| ReturnValue    | `Entry`     | The entry for the given key | 

#### Edge cases

- Will revert with `Registry__RollupNotRegistered(address rollup)` if `msg.sender` is not registered as a rollup on the [`Registry`](./registry.md).
- Will revert with `Inbox__InvalidVersion(uint256 entry, uint256 rollup)` if the rollup version does not match the version specified in the message.
- Will revert with `Inbox__PastDeadline()` if the message deadline has passed.
- Will revert with `Inbox__NothingToConsume(bytes32 entryKey)` if the message does not exist.

## `withdrawFees()`

Will claim the fees that has accrued to the `msg.sender` from consuming messages. 

Let the sequencer withdraw fees from the inbox.

#include_code inbox_withdraw_fees l1-contracts/src/core/interfaces/messagebridge/IInbox.sol solidity

#### Edge cases

- Will revert with `Inbox__FailedToWithdrawFees()` if the transfer call fails.

## `get()`
Retrieves the `entry` for a given message. The entry contains fee, number of occurrences, deadline and version information. 

#include_code inbox_get l1-contracts/src/core/interfaces/messagebridge/IInbox.sol solidity

| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_entryKey`    | `bytes32`   | The entry key (message hash) |
| ReturnValue    | `Entry`     | The entry object for the given key | 

#### Edge cases
- Will revert with `Inbox__NothingToConsume(bytes32 entryKey)` if the message does not exist.


## `contains()`
Returns whether the key exists in the inbox.

#include_code inbox_contains l1-contracts/src/core/interfaces/messagebridge/IInbox.sol solidity

| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_entryKey`    | `bytes32`   | The entry key (message hash)|
| ReturnValue    | `bool`   | True if contained, false otherwise| 

## `computeEntryKey()`
Computes the hash of a message.

#include_code inbox_compute_entry_key l1-contracts/src/core/interfaces/messagebridge/IInbox.sol solidity

| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_message`     | `L1ToL2Msg` | The message to compute hash for |
| ReturnValue    | `bytes32`   | The hash of the message | 

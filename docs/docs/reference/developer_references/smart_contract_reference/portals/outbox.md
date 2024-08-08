---
title: Outbox
tags: [portals, contracts]
---

The `Outbox` is a contract deployed on L1 that handles message passing from the rollup and to L1.

**Links**: [Interface (GitHub link)](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol), [Implementation (GitHub link)](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/messagebridge/Outbox.sol).

## `insert()`

Inserts the root of a merkle tree containing all of the L2 to L1 messages in a block specified by _l2BlockNumber.

#include_code outbox_insert l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol solidity


| Name           | Type    | Description |
| -------------- | ------- | ----------- |
| `_l2BlockNumber` | `uint256` | The L2 Block Number in which the L2 to L1 messages reside |
| `_root` | `bytes32` | The merkle root of the tree where all the L2 to L1 messages are leaves |
| `_minHeight` | `uint256` | The minimum height of the merkle tree that the root corresponds to |

#### Edge cases

- Will revert with `Outbox__Unauthorized()` if `msg.sender != ROLLUP_CONTRACT`. 
- Will revert with `Errors.Outbox__RootAlreadySetAtBlock(uint256 l2BlockNumber)` if the root for the specific block has already been set.
- Will revert with `Errors.Outbox__InsertingInvalidRoot()` if the rollup is trying to insert bytes32(0) as the root.

## `consume()`

Allows a recipient to consume a message from the `Outbox`.

#include_code outbox_consume l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol solidity


| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_message`     | `L2ToL1Msg` | The L2 to L1 message we want to consume |
| `_l2BlockNumber`     | `uint256` | The block number specifying the block that contains the message we want to consume |
| `_leafIndex`     | `uint256` | The index inside the merkle tree where the message is located |
| `_path`     | `bytes32[]` | The sibling path used to prove inclusion of the message, the _path length directly depends |

#### Edge cases

- Will revert with `Outbox__InvalidRecipient(address expected, address actual);` if `msg.sender != _message.recipient.actor`. 
- Will revert with `Outbox__InvalidChainId()` if `block.chainid != _message.recipient.chainId`.
- Will revert with `Outbox__NothingToConsumeAtBlock(uint256 l2BlockNumber)` if the root for the block has not been set yet.
- Will revert with `Outbox__AlreadyNullified(uint256 l2BlockNumber, uint256 leafIndex)` if the message at leafIndex for the block has already been consumed.
- Will revert with `Outbox__InvalidPathLength(uint256 expected, uint256 actual)` if the supplied height is less than the existing minimum height of the L2 to L1 message tree, or the supplied height is greater than the maximum (minimum height + log2(maximum messages)).
- Will revert with `MerkleLib__InvalidRoot(bytes32 expected, bytes32 actual, bytes32 leaf, uint256 leafIndex)` if unable to verify the message existence in the tree. It returns the message as a leaf, as well as the index of the leaf to expose more info about the error.


## `hasMessageBeenConsumedAtBlockAndIndex()`

Checks to see if an index of the L2 to L1 message tree for a specific block has been consumed.

#include_code outbox_has_message_been_consumed_at_block_and_index l1-contracts/src/core/interfaces/messagebridge/IOutbox.sol solidity


| Name           | Type        | Description |
| -------------- | -------     | ----------- |
| `_l2BlockNumber`     | `uint256` | The block number specifying the block that contains the index of the message we want to check |
| `_leafIndex`     | `uint256` | The index of the message inside the merkle tree |

#### Edge cases

- This function does not throw. Out-of-bounds access is considered valid, but will always return false.

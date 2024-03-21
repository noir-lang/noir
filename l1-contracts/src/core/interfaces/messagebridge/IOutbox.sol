// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {DataStructures} from "../../libraries/DataStructures.sol";

/**
 * @title IOutbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to consume L2 -> L1 messages. Messages are inserted by the Rollup
 * and will be consumed by the portal contracts.
 */
interface IOutbox {
  event RootAdded(uint256 indexed l2BlockNumber, bytes32 indexed root, uint256 height);
  event MessageConsumed(
    uint256 indexed l2BlockNumber,
    bytes32 indexed root,
    bytes32 indexed messageHash,
    uint256 leafIndex
  );

  // docs:start:outbox_insert
  /**
   * @notice Inserts the root of a merkle tree containing all of the L2 to L1 messages in
   * a block specified by _l2BlockNumber.
   * @dev Only callable by the rollup contract
   * @dev Emits `RootAdded` upon inserting the root successfully
   * @param _l2BlockNumber - The L2 Block Number in which the L2 to L1 messages reside
   * @param _root - The merkle root of the tree where all the L2 to L1 messages are leaves
   * @param _height - The height of the merkle tree that the root corresponds to
   */
  function insert(uint256 _l2BlockNumber, bytes32 _root, uint256 _height) external;
  // docs:end:outbox_insert

  // docs:start:outbox_consume
  /**
   * @notice Consumes an entry from the Outbox
   * @dev Only useable by portals / recipients of messages
   * @dev Emits `MessageConsumed` when consuming messages
   * @param _message - The L2 to L1 message
   * @param _l2BlockNumber - The block number specifying the block that contains the message we want to consume
   * @param _leafIndex - The index inside the merkle tree where the message is located
   * @param _path - The sibling path used to prove inclusion of the message, the _path length directly depends
   * on the total amount of L2 to L1 messages in the block. i.e. the length of _path is equal to the depth of the
   * L1 to L2 message tree.
   */
  function consume(
    DataStructures.L2ToL1Msg calldata _message,
    uint256 _l2BlockNumber,
    uint256 _leafIndex,
    bytes32[] calldata _path
  ) external;
  // docs:end:outbox_consume

  // docs:start:outbox_has_message_been_consumed_at_block_and_index
  /**
   * @notice Checks to see if an index of the L2 to L1 message tree for a specific block has been consumed
   * @dev - This function does not throw. Out-of-bounds access is considered valid, but will always return false
   * @param _l2BlockNumber - The block number specifying the block that contains the index of the message we want to check
   * @param _leafIndex - The index of the message inside the merkle tree
   */
  function hasMessageBeenConsumedAtBlockAndIndex(uint256 _l2BlockNumber, uint256 _leafIndex)
    external
    view
    returns (bool);
  // docs:end:outbox_has_message_been_consumed_at_block_and_index
}

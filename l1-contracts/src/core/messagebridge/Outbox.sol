// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {DataStructures} from "../libraries/DataStructures.sol";
import {Errors} from "../libraries/Errors.sol";
import {MerkleLib} from "../libraries/MerkleLib.sol";
import {Hash} from "../libraries/Hash.sol";
import {IOutbox} from "../interfaces/messagebridge/IOutbox.sol";

import {Rollup} from "../Rollup.sol";

/**
 * @title Outbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to consume L2 -> L1 messages. Messages are inserted by the Rollup
 * and will be consumed by the portal contracts.
 */
contract Outbox is IOutbox {
  using Hash for DataStructures.L2ToL1Msg;

  struct RootData {
    // This is the outhash specified by header.globalvariables.outHash of any given block.
    bytes32 root;
    uint256 minHeight;
    mapping(uint256 => bool) nullified;
  }

  Rollup public immutable ROLLUP;
  mapping(uint256 l2BlockNumber => RootData) internal roots;

  constructor(address _rollup) {
    ROLLUP = Rollup(_rollup);
  }

  /**
   * @notice Inserts the root of a merkle tree containing all of the L2 to L1 messages in a block
   *
   * @dev Only callable by the rollup contract
   * @dev Emits `RootAdded` upon inserting the root successfully
   *
   * @param _l2BlockNumber - The L2 Block Number in which the L2 to L1 messages reside
   * @param _root - The merkle root of the tree where all the L2 to L1 messages are leaves
   * @param _minHeight - The min height of the merkle tree that the root corresponds to
   */
  function insert(uint256 _l2BlockNumber, bytes32 _root, uint256 _minHeight)
    external
    override(IOutbox)
  {
    if (msg.sender != address(ROLLUP)) {
      revert Errors.Outbox__Unauthorized();
    }

    if (_root == bytes32(0)) {
      revert Errors.Outbox__InsertingInvalidRoot();
    }

    roots[_l2BlockNumber].root = _root;
    roots[_l2BlockNumber].minHeight = _minHeight;

    emit RootAdded(_l2BlockNumber, _root, _minHeight);
  }

  /**
   * @notice Consumes an entry from the Outbox
   *
   * @dev Only useable by portals / recipients of messages
   * @dev Emits `MessageConsumed` when consuming messages
   *
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
  ) external override(IOutbox) {
    if (_l2BlockNumber >= ROLLUP.provenBlockCount()) {
      revert Errors.Outbox__BlockNotProven(_l2BlockNumber);
    }

    if (msg.sender != _message.recipient.actor) {
      revert Errors.Outbox__InvalidRecipient(_message.recipient.actor, msg.sender);
    }

    if (block.chainid != _message.recipient.chainId) {
      revert Errors.Outbox__InvalidChainId();
    }

    RootData storage rootData = roots[_l2BlockNumber];

    bytes32 blockRoot = rootData.root;

    if (blockRoot == 0) {
      revert Errors.Outbox__NothingToConsumeAtBlock(_l2BlockNumber);
    }

    if (rootData.nullified[_leafIndex]) {
      revert Errors.Outbox__AlreadyNullified(_l2BlockNumber, _leafIndex);
    }
    // TODO(#7218): We will eventually move back to a balanced tree and constrain the path length
    // to be equal to height - for now we just check the min

    // Min height = height of rollup layers
    // The smallest num of messages will require a subtree of height 1
    uint256 minHeight = rootData.minHeight;
    if (minHeight > _path.length) {
      revert Errors.Outbox__InvalidPathLength(minHeight, _path.length);
    }

    bytes32 messageHash = _message.sha256ToField();

    MerkleLib.verifyMembership(_path, messageHash, _leafIndex, blockRoot);

    rootData.nullified[_leafIndex] = true;

    emit MessageConsumed(_l2BlockNumber, blockRoot, messageHash, _leafIndex);
  }

  /**
   * @notice Checks to see if an index of the L2 to L1 message tree for a specific block has been consumed
   *
   * @dev - This function does not throw. Out-of-bounds access is considered valid, but will always return false
   *
   * @param _l2BlockNumber - The block number specifying the block that contains the index of the message we want to check
   * @param _leafIndex - The index of the message inside the merkle tree
   *
   * @return bool - True if the message has been consumed, false otherwise
   */
  function hasMessageBeenConsumedAtBlockAndIndex(uint256 _l2BlockNumber, uint256 _leafIndex)
    external
    view
    override(IOutbox)
    returns (bool)
  {
    return roots[_l2BlockNumber].nullified[_leafIndex];
  }

  /**
   * @notice  Fetch the root data for a given block number
   *          Returns (0, 0) if the block is not proven
   *
   * @param _l2BlockNumber - The block number to fetch the root data for
   *
   * @return root - The root of the merkle tree containing the L2 to L1 messages
   * @return minHeight - The min height for the the merkle tree that the root corresponds to
   */
  function getRootData(uint256 _l2BlockNumber)
    external
    view
    override(IOutbox)
    returns (bytes32 root, uint256 minHeight)
  {
    if (_l2BlockNumber >= ROLLUP.provenBlockCount()) {
      return (bytes32(0), 0);
    }
    RootData storage rootData = roots[_l2BlockNumber];
    return (rootData.root, rootData.minHeight);
  }
}

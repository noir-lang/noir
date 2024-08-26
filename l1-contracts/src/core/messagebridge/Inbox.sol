// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IInbox} from "../interfaces/messagebridge/IInbox.sol";

// Libraries
import {Constants} from "../libraries/ConstantsGen.sol";
import {DataStructures} from "../libraries/DataStructures.sol";
import {Errors} from "../libraries/Errors.sol";
import {Hash} from "../libraries/Hash.sol";

import {FrontierLib} from "./frontier_tree/FrontierLib.sol";

/**
 * @title Inbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to pass messages into the rollup, e.g., L1 -> L2 messages.
 */
contract Inbox is IInbox {
  using Hash for DataStructures.L1ToL2Msg;
  using FrontierLib for FrontierLib.Forest;
  using FrontierLib for FrontierLib.Tree;

  address public immutable ROLLUP;

  uint256 internal immutable HEIGHT;
  uint256 internal immutable SIZE;
  bytes32 internal immutable EMPTY_ROOT; // The root of an empty frontier tree

  // Number of a tree which is currently being filled
  uint256 public inProgress = Constants.INITIAL_L2_BLOCK_NUM + 1;

  // Practically immutable value as we only set it in the constructor.
  FrontierLib.Forest internal forest;

  mapping(uint256 blockNumber => FrontierLib.Tree tree) public trees;

  constructor(address _rollup, uint256 _height) {
    ROLLUP = _rollup;

    HEIGHT = _height;
    SIZE = 2 ** _height;

    forest.initialize(_height);
    EMPTY_ROOT = trees[inProgress].root(forest, HEIGHT, SIZE);
  }

  /**
   * @notice Inserts a new message into the Inbox
   *
   * @dev Emits `MessageSent` with data for easy access by the sequencer
   *
   * @param _recipient - The recipient of the message
   * @param _content - The content of the message (application specific)
   * @param _secretHash - The secret hash of the message (make it possible to hide when a specific message is consumed on L2)
   *
   * @return Hash of the sent message.
   */
  function sendL2Message(
    DataStructures.L2Actor memory _recipient,
    bytes32 _content,
    bytes32 _secretHash
  ) external override(IInbox) returns (bytes32) {
    if (uint256(_recipient.actor) > Constants.MAX_FIELD_VALUE) {
      revert Errors.Inbox__ActorTooLarge(_recipient.actor);
    }
    if (uint256(_content) > Constants.MAX_FIELD_VALUE) {
      revert Errors.Inbox__ContentTooLarge(_content);
    }
    if (uint256(_secretHash) > Constants.MAX_FIELD_VALUE) {
      revert Errors.Inbox__SecretHashTooLarge(_secretHash);
    }

    FrontierLib.Tree storage currentTree = trees[inProgress];

    if (currentTree.isFull(SIZE)) {
      inProgress += 1;
      currentTree = trees[inProgress];
    }

    DataStructures.L1ToL2Msg memory message = DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor(msg.sender, block.chainid),
      recipient: _recipient,
      content: _content,
      secretHash: _secretHash
    });

    bytes32 leaf = message.sha256ToField();
    uint256 index = currentTree.insertLeaf(leaf);
    emit MessageSent(inProgress, index, leaf);

    return leaf;
  }

  /**
   * @notice Consumes the current tree, and starts a new one if needed
   *
   * @dev Only callable by the rollup contract
   * @dev In the first iteration we return empty tree root because first block's messages tree is always
   * empty because there has to be a 1 block lag to prevent sequencer DOS attacks
   *
   * @param _toConsume - The block number to consume
   *
   * @return The root of the consumed tree
   */
  function consume(uint256 _toConsume) external override(IInbox) returns (bytes32) {
    if (msg.sender != ROLLUP) {
      revert Errors.Inbox__Unauthorized();
    }

    if (_toConsume >= inProgress) {
      revert Errors.Inbox__MustBuildBeforeConsume();
    }

    bytes32 root = EMPTY_ROOT;
    if (_toConsume > Constants.INITIAL_L2_BLOCK_NUM) {
      root = trees[_toConsume].root(forest, HEIGHT, SIZE);
    }

    // If we are "catching up" we skip the tree creation as it is already there
    if (_toConsume + 1 == inProgress) {
      inProgress += 1;
    }

    return root;
  }

  function getRoot(uint256 _blockNumber) external view override(IInbox) returns (bytes32) {
    return trees[_blockNumber].root(forest, HEIGHT, SIZE);
  }
}

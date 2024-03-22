// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IFrontier} from "../interfaces/messagebridge/IFrontier.sol";
import {IRegistry} from "../interfaces/messagebridge/IRegistry.sol";
import {IInbox} from "../interfaces/messagebridge/IInbox.sol";

// Libraries
import {Constants} from "../libraries/ConstantsGen.sol";
import {DataStructures} from "../libraries/DataStructures.sol";
import {Errors} from "../libraries/Errors.sol";
import {Hash} from "../libraries/Hash.sol";

// Contracts
import {FrontierMerkle} from "./frontier_tree/Frontier.sol";

/**
 * @title Inbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to pass messages into the rollup, e.g., L1 -> L2 messages.
 */
contract Inbox is IInbox {
  using Hash for DataStructures.L1ToL2Msg;

  address public immutable ROLLUP;

  uint256 internal immutable HEIGHT;
  uint256 internal immutable SIZE;
  bytes32 internal immutable EMPTY_ROOT; // The root of an empty frontier tree

  // Number of a tree which is ready to be consumed
  uint256 public toConsume = Constants.INITIAL_L2_BLOCK_NUM;
  // Number of a tree which is currently being filled
  uint256 public inProgress = Constants.INITIAL_L2_BLOCK_NUM + 1;

  mapping(uint256 blockNumber => IFrontier tree) internal trees;

  constructor(address _rollup, uint256 _height) {
    ROLLUP = _rollup;

    HEIGHT = _height;
    SIZE = 2 ** _height;

    // We deploy the first tree
    IFrontier firstTree = IFrontier(new FrontierMerkle(_height));
    trees[inProgress] = firstTree;

    EMPTY_ROOT = firstTree.root();
  }

  /**
   * @notice Inserts a new message into the Inbox
   * @dev Emits `MessageSent` with data for easy access by the sequencer
   * @param _recipient - The recipient of the message
   * @param _content - The content of the message (application specific)
   * @param _secretHash - The secret hash of the message (make it possible to hide when a specific message is consumed on L2)
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

    IFrontier currentTree = trees[inProgress];
    if (currentTree.isFull()) {
      inProgress += 1;
      currentTree = IFrontier(new FrontierMerkle(HEIGHT));
      trees[inProgress] = currentTree;
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
   * @dev Only callable by the rollup contract
   * @dev In the first iteration we return empty tree root because first block's messages tree is always
   * empty because there has to be a 1 block lag to prevent sequencer DOS attacks
   * @return The root of the consumed tree
   */
  function consume() external override(IInbox) returns (bytes32) {
    if (msg.sender != ROLLUP) {
      revert Errors.Inbox__Unauthorized();
    }

    bytes32 root = EMPTY_ROOT;
    if (toConsume > Constants.INITIAL_L2_BLOCK_NUM) {
      root = trees[toConsume].root();
    }

    // If we are "catching up" we skip the tree creation as it is already there
    if (toConsume + 1 == inProgress) {
      inProgress += 1;
      trees[inProgress] = IFrontier(new FrontierMerkle(HEIGHT));
    }

    toConsume += 1;

    return root;
  }
}

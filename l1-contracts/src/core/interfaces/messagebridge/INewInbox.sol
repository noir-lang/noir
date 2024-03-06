// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DataStructures} from "../../libraries/DataStructures.sol";

/**
 * @title Inbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to pass messages into the rollup, e.g., L1 -> L2 messages.
 */
// TODO: rename to IInbox once all the pieces of the new message model are in place.
interface INewInbox {
  event LeafInserted(uint256 indexed blockNumber, uint256 index, bytes32 value);

  /**
   * @notice Inserts a new message into the Inbox
   * @dev Emits `LeafInserted` with data for easy access by the sequencer
   * @param _recipient - The recipient of the message
   * @param _content - The content of the message (application specific)
   * @param _secretHash - The secret hash of the message (make it possible to hide when a specific message is consumed on L2)
   * @return The key of the message in the set
   */
  function sendL2Message(
    DataStructures.L2Actor memory _recipient,
    bytes32 _content,
    bytes32 _secretHash
  ) external returns (bytes32);

  /**
   * @notice Consumes the current tree, and starts a new one if needed
   * @dev Only callable by the rollup contract
   * @dev In the first iteration we return empty tree root because first block's messages tree is always
   * empty because there has to be a 1 block lag to prevent sequencer DOS attacks
   * @return The root of the consumed tree
   */
  function consume() external returns (bytes32);
}

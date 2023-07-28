// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";

/**
 * @title Inbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to pass messages into the rollup, e.g., L1 -> L2 messages.
 */
interface IInbox {
  event MessageAdded(
    bytes32 indexed entryKey,
    address indexed sender,
    bytes32 indexed recipient,
    uint256 senderChainId,
    uint256 recipientVersion,
    uint32 deadline,
    uint64 fee,
    bytes32 content,
    bytes32 secretHash
  );

  event L1ToL2MessageCancelled(bytes32 indexed entryKey);

  /**
   * @notice Inserts an entry into the Inbox
   * @dev Will emit `MessageAdded` with data for easy access by the sequencer
   * @dev msg.value - The fee provided to sequencer for including the entry
   * @param _recipient - The recipient of the entry
   * @param _deadline - The deadline to consume a message. Only after it, can a message be cancelled.
   * @param _content - The content of the entry (application specific)
   * @param _secretHash - The secret hash of the entry (make it possible to hide when a specific entry is consumed on L2)
   * @return The key of the entry in the set
   */
  function sendL2Message(
    DataStructures.L2Actor memory _recipient,
    uint32 _deadline,
    bytes32 _content,
    bytes32 _secretHash
  ) external payable returns (bytes32);

  /**
   * @notice Cancel a pending L2 message
   * @dev Will revert if the deadline have not been crossed - message only cancellable past the deadline
   *  so it cannot be yanked away while the sequencer is building a block including it
   * @dev Must be called by portal that inserted the entry
   * @param _message - The content of the entry (application specific)
   * @param _feeCollector - The address to receive the "fee"
   * @return entryKey - The key of the entry removed
   */
  function cancelL2Message(DataStructures.L1ToL2Msg memory _message, address _feeCollector)
    external
    returns (bytes32 entryKey);

  /**
   * @notice Batch consumes entries from the Inbox
   * @dev Only callable by the rollup contract
   * @dev Will revert if the message is already past deadline
   * @param _entryKeys - Array of entry keys (hash of the messages)
   * @param _feeCollector - The address to receive the "fee"
   */
  function batchConsume(bytes32[] memory _entryKeys, address _feeCollector) external;

  /**
   * @notice Withdraws fees accrued by the sequencer
   */
  function withdrawFees() external;

  /**
   * @notice Fetch an entry
   * @param _entryKey - The key to lookup
   * @return The entry matching the provided key
   */
  function get(bytes32 _entryKey) external view returns (DataStructures.Entry memory);

  /**
   * @notice Check if entry exists
   * @param _entryKey - The key to lookup
   * @return True if entry exists, false otherwise
   */
  function contains(bytes32 _entryKey) external view returns (bool);

  /// @notice Given a message, computes an entry key for the Inbox
  function computeEntryKey(DataStructures.L1ToL2Msg memory _message)
    external
    pure
    returns (bytes32);
}

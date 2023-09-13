// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DataStructures} from "../../libraries/DataStructures.sol";

/**
 * @title IOutbox
 * @author Aztec Labs
 * @notice Lives on L1 and is used to consume L2 -> L1 messages. Messages are inserted by the rollup contract
 * and will be consumed by the portal contracts.
 */
interface IOutbox {
  // to make it easier for portal to know when to consume the message.
  event MessageAdded(bytes32 indexed entryKey);

  event MessageConsumed(bytes32 indexed entryKey, address indexed recipient);

  // docs:start:outbox_compute_entry_key
  /**
   * @notice Computes an entry key for the Outbox
   * @param _message - The L2 to L1 message
   * @return The key of the entry in the set
   */
  function computeEntryKey(DataStructures.L2ToL1Msg memory _message) external returns (bytes32);
  // docs:end:outbox_compute_entry_key

  // docs:start:outbox_send_l1_msg
  /**
   * @notice Inserts an array of entries into the Outbox
   * @dev Only callable by the rollup contract
   * @param _entryKeys - Array of entry keys (hash of the message) - computed by the L2 counterpart and sent to L1 via rollup block
   */
  function sendL1Messages(bytes32[] memory _entryKeys) external;
  // docs:end:outbox_send_l1_msg

  // docs:start:outbox_consume
  /**
   * @notice Consumes an entry from the Outbox
   * @dev Only meaningfully callable by portals, otherwise should never hit an entry
   * @dev Emits the `MessageConsumed` event when consuming messages
   * @param _message - The L2 to L1 message
   * @return entryKey - The key of the entry removed
   */
  function consume(DataStructures.L2ToL1Msg memory _message) external returns (bytes32 entryKey);
  // docs:end:outbox_consume

  // docs:start:outbox_get
  /**
   * @notice Fetch an entry
   * @param _entryKey - The key to lookup
   * @return The entry matching the provided key
   */
  function get(bytes32 _entryKey) external view returns (DataStructures.Entry memory);
  // docs:end:outbox_get

  // docs:start:outbox_contains
  /**
   * @notice Check if entry exists
   * @param _entryKey - The key to lookup
   * @return True if entry exists, false otherwise
   */
  function contains(bytes32 _entryKey) external view returns (bool);
  // docs:end:outbox_contains
}

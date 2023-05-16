// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DataStructures} from "../../libraries/DataStructures.sol";

/**
 * @title IMessageBox
 * @author Aztec Labs
 * @notice Data structure used in both Inbox and Outbox for keeping track of entries
 * Implements a multi-set storing the multiplicity (count for easy reading) at the entry.
 */
interface IMessageBox {
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
}

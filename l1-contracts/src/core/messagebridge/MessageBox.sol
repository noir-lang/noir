// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {IRegistry} from "@aztec/core/interfaces/messagebridge/IRegistry.sol";
import {IMessageBox} from "@aztec/core/interfaces/messagebridge/IMessageBox.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";

/**
 * @title MessageBox
 * @author Aztec Labs
 * @notice Data structure used in both Inbox and Outbox for keeping track of entries
 * Implements a multi-set storing the multiplicity (count for easy reading) at the entry.
 */
abstract contract MessageBox is IMessageBox {
  error MessageBox__Unauthorized();
  error MessageBox__IncompatibleEntryArguments(
    bytes32 entryKey,
    uint64 storedFee,
    uint64 feePassed,
    uint32 storedDeadline,
    uint32 deadlinePassed
  );
  error MessageBox__NothingToConsume(bytes32 entryKey);

  // Prime field order
  uint256 internal constant P =
    21888242871839275222246405745257275088548364400416034343698204186575808495617;

  IRegistry immutable REGISTRY;

  mapping(bytes32 entryKey => DataStructures.Entry entry) internal entries;

  modifier onlyRollup() {
    if (msg.sender != address(REGISTRY.getRollup())) revert MessageBox__Unauthorized();
    _;
  }

  constructor(address _registry) {
    REGISTRY = IRegistry(_registry);
  }

  /**
   * @notice Inserts an entry into the multi-set
   * @param _entryKey - The key to insert
   * @param _fee - The fee provided to sequencer for including in the inbox. 0 if Oubox (as not applicable).
   * @param _deadline - The deadline to consume a message. Only after it, can a message be cancelled.
   */
  function _insert(bytes32 _entryKey, uint64 _fee, uint32 _deadline) internal {
    // since entryKey is a hash of the message, _fee and `deadline` should always be the same
    // as such, there is no need to update these vars. Yet adding an if statement breaks
    // the slot packing and increases gas. So we leave it as it is.
    DataStructures.Entry memory entry = entries[_entryKey];
    if (
      (entry.fee != 0 && entry.fee != _fee) || (entry.deadline != 0 && entry.deadline != _deadline)
    ) {
      // this should never happen as it is trying to overwrite `fee` and `deadline` with different values
      // even though the entryKey (a hash) is the same! Pass all arguments to the error message for debugging.
      revert MessageBox__IncompatibleEntryArguments(
        _entryKey, entry.fee, _fee, entry.deadline, _deadline
      );
    }
    entry.count += 1;
    entry.fee = _fee;
    entry.deadline = _deadline;
    entries[_entryKey] = entry;
  }

  /**
   * @notice Consumed an entry if possible, reverts if nothing to consume
   * For multiplicity > 1, will consume one element
   * @param _entryKey - The key to consume
   */
  function _consume(bytes32 _entryKey) internal {
    DataStructures.Entry storage entry = entries[_entryKey];
    if (entry.count == 0) revert MessageBox__NothingToConsume(_entryKey);
    entry.count--;
  }

  /**
   * @notice Fetch an entry
   * @param _entryKey - The key to lookup
   * @return The entry matching the provided key
   */
  function get(bytes32 _entryKey) public view returns (DataStructures.Entry memory) {
    DataStructures.Entry memory entry = entries[_entryKey];
    if (entry.count == 0) revert MessageBox__NothingToConsume(_entryKey);
    return entry;
  }

  /**
   * @notice Check if entry exists
   * @param _entryKey - The key to lookup
   * @return True if entry exists, false otherwise
   */
  function contains(bytes32 _entryKey) public view returns (bool) {
    return entries[_entryKey].count > 0;
  }
}

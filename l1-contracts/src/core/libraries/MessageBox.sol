// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Errors} from "./Errors.sol";
import {DataStructures} from "./DataStructures.sol";

/**
 * @title MessageBox
 * @author Aztec
 * @notice Implements a multi-set of entries (DataStructures.Entry)
 */
library MessageBox {
  function insert(
    mapping(bytes32 entryKey => DataStructures.Entry entry) storage self,
    bytes32 _entryKey,
    uint64 _fee,
    uint32 _deadline,
    function(
    bytes32,
    uint64,
    uint64,
    uint32,
    uint32 
    ) pure _err
  ) internal {
    DataStructures.Entry memory entry = self[_entryKey];
    if (
      (entry.fee != 0 && entry.fee != _fee) || (entry.deadline != 0 && entry.deadline != _deadline)
    ) {
      // this should never happen as it is trying to overwrite `fee` and `deadline` with different values
      // even though the entryKey (a hash) is the same! Pass all arguments to the error message for debugging.
      _err(_entryKey, entry.fee, _fee, entry.deadline, _deadline);
    }
    entry.count += 1;
    entry.fee = _fee;
    entry.deadline = _deadline;
    self[_entryKey] = entry;
  }

  function contains(
    mapping(bytes32 entryKey => DataStructures.Entry entry) storage self,
    bytes32 _entryKey
  ) internal view returns (bool) {
    return self[_entryKey].count > 0;
  }
  /**
   * @notice Fetch an entry
   * @param _entryKey - The key to lookup
   * @return The entry matching the provided key
   */

  function get(
    mapping(bytes32 entryKey => DataStructures.Entry entry) storage self,
    bytes32 _entryKey,
    function(bytes32) view _err
  ) internal view returns (DataStructures.Entry memory) {
    DataStructures.Entry memory entry = self[_entryKey];
    if (entry.count == 0) _err(_entryKey);
    return entry;
  }

  /**
   * @notice Consumed an entry if possible, reverts if nothing to consume
   * For multiplicity > 1, will consume one element
   * @param _entryKey - The key to consume
   * @param _err - A function taking _entryKey as param that should contain a revert
   */
  function consume(
    mapping(bytes32 entryKey => DataStructures.Entry entry) storage self,
    bytes32 _entryKey,
    function(bytes32) view _err
  ) internal {
    DataStructures.Entry storage entry = self[_entryKey];
    if (entry.count == 0) _err(_entryKey);
    entry.count--;
  }
}

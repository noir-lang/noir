// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Libraries
import {DataStructures} from "./DataStructures.sol";
import {Errors} from "./Errors.sol";

/**
 * @title MessageBox
 * @author Aztec Labs
 * @notice Library that implements multi-set logic for a mapping of entries (DataStructures.Entry)
 * Allows for inserting, consuming, checking existence and fetching entries
 * @dev This library is used by `Inbox` and `Outbox` to store messages
 * @dev Allow passing of `_err` functions to allow for custom error messages dependent on the context of use
 */
library MessageBox {
  /**
   * @notice Inserts an entry into the MessageBox (multi-set)
   * @dev Will increment the count of the entry if it already exists
   * Will revert if the entry already exists with different fee or deadline
   * @param _self - The storage mapping containing all entries
   * @param _entryKey - The key to insert
   * @param _fee - The fee to insert
   * @param _deadline - The deadline to insert
   * @param _err - A function taking _entryKey, _fee, _deadline as params that MUST revert with a error reason
   * @dev The _err function is passed as a param to allow for custom error messages dependent on the context of use
   * We use it to allow `Inbox` and `Outbox` to throw distinct errors
   */
  function insert(
    mapping(bytes32 entryKey => DataStructures.Entry entry) storage _self,
    bytes32 _entryKey,
    uint64 _fee,
    uint32 _version,
    uint32 _deadline,
    function(
    bytes32,
    uint64,
    uint64,
    uint32,
    uint32,
    uint32,
    uint32 
    ) pure _err
  ) internal {
    DataStructures.Entry memory entry = _self[_entryKey];
    if (
      (entry.fee != 0 && entry.fee != _fee) || (entry.deadline != 0 && entry.deadline != _deadline)
        || (entry.version != 0 && entry.version != _version)
    ) {
      // this should never happen as it is trying to overwrite `fee`, `version` and `deadline` with different values
      // even though the entryKey (a hash) is the same! Pass all arguments to the error message for debugging.
      _err(_entryKey, entry.fee, _fee, entry.version, _version, entry.deadline, _deadline);
    }
    entry.count += 1;
    entry.fee = _fee;
    entry.version = _version;
    entry.deadline = _deadline;
    _self[_entryKey] = entry;
  }

  /**
   * @notice Consume an entry if possible, reverts if nothing to consume
   * @dev For multiplicity > 1, will consume one element
   * @param _self - The storage mapping containing all entries
   * @param _entryKey - The key to consume
   * @param _err - A function taking _entryKey as param that MUST revert with a error reason
   * @dev The _err function is passed as a param to allow for custom error messages dependent on the context of use
   * We use it to allow `Inbox` and `Outbox` to throw distinct errors
   */
  function consume(
    mapping(bytes32 entryKey => DataStructures.Entry entry) storage _self,
    bytes32 _entryKey,
    function(bytes32) view _err
  ) internal {
    DataStructures.Entry storage entry = _self[_entryKey];
    if (entry.count == 0) _err(_entryKey);
    entry.count--;
  }

  /**
   * @notice Check if an entry exists
   * @param _self - The storage mapping containing all entries
   * @param _entryKey - The key to lookup
   * @return True if the entry exists, false otherwise
   */
  function contains(
    mapping(bytes32 entryKey => DataStructures.Entry entry) storage _self,
    bytes32 _entryKey
  ) internal view returns (bool) {
    return _self[_entryKey].count > 0;
  }

  /**
   * @notice Fetch an entry
   * @dev Will revert if the entry does not exist
   * @param _self - The storage mapping containing all entries
   * @param _entryKey - The key to lookup
   * @param _err - A function taking _entryKey as param that MUST revert with a error reason
   * @dev The _err function is passed as a param to allow for custom error messages dependent on the context of use
   * We use it to allow `Inbox` and `Outbox` to throw distinct errors
   * @return The entry matching the provided key
   */
  function get(
    mapping(bytes32 entryKey => DataStructures.Entry entry) storage _self,
    bytes32 _entryKey,
    function(bytes32) view _err
  ) internal view returns (DataStructures.Entry memory) {
    DataStructures.Entry memory entry = _self[_entryKey];
    if (entry.count == 0) _err(_entryKey);
    return entry;
  }
}

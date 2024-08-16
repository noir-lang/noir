// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Ownable} from "@oz/access/Ownable.sol";

// Interfaces
import {IRegistry} from "../interfaces/messagebridge/IRegistry.sol";
import {IRollup} from "../interfaces/IRollup.sol";

// Libraries
import {DataStructures} from "../libraries/DataStructures.sol";
import {Errors} from "../libraries/Errors.sol";

/**
 * @title Registry
 * @author Aztec Labs
 * @notice Keeps track of addresses of current rollup and historical addresses.
 */
contract Registry is IRegistry, Ownable {
  uint256 public override(IRegistry) numberOfVersions;

  DataStructures.RegistrySnapshot internal currentSnapshot;
  mapping(uint256 version => DataStructures.RegistrySnapshot snapshot) internal snapshots;
  mapping(address rollup => uint256 version) internal rollupToVersion;

  constructor(address _owner) Ownable(_owner) {
    // Inserts a "dead" rollup at version 0
    // This is simply done to make first version 1, which fits better with the rest of the system
    _upgrade(address(0xdead));
  }

  /**
   * @notice Returns the rollup contract
   * @return The rollup contract (of type IRollup)
   */
  function getRollup() external view override(IRegistry) returns (IRollup) {
    return IRollup(currentSnapshot.rollup);
  }

  /**
   * @notice Returns the version for a specific rollup contract or reverts if not listed
   * @param _rollup - The address of the rollup contract
   * @return The version of the rollup contract
   */
  function getVersionFor(address _rollup) external view override(IRegistry) returns (uint256) {
    (uint256 version, bool exists) = _getVersionFor(_rollup);
    if (!exists) revert Errors.Registry__RollupNotRegistered(_rollup);
    return version;
  }

  /**
   * @notice Returns whther the rollup is registered
   * @param _rollup - The address of the rollup contract
   * @return Whether the rollup is registered
   */
  function isRollupRegistered(address _rollup) external view override(IRegistry) returns (bool) {
    (, bool exists) = _getVersionFor(_rollup);
    return exists;
  }

  /**
   * @notice Fetches a snapshot of the registry indicated by `version`
   * @dev the version is 0 indexed, so the first snapshot is version 0.
   * @param _version - The version of the rollup to return (i.e. which snapshot)
   * @return the snapshot
   */
  function getSnapshot(uint256 _version)
    external
    view
    override(IRegistry)
    returns (DataStructures.RegistrySnapshot memory)
  {
    return snapshots[_version];
  }

  /**
   * @notice Returns the current snapshot of the registry
   * @return The current snapshot
   */
  function getCurrentSnapshot()
    external
    view
    override(IRegistry)
    returns (DataStructures.RegistrySnapshot memory)
  {
    return currentSnapshot;
  }

  /**
   * @notice Creates a new snapshot of the registry
   *
   * @dev Only callable by the owner
   * @dev Reverts if the rollup is already registered
   *
   * @param _rollup - The address of the rollup contract
   * @return The version of the new snapshot
   */
  function upgrade(address _rollup) public override(IRegistry) onlyOwner returns (uint256) {
    return _upgrade(_rollup);
  }

  function _upgrade(address _rollup) internal returns (uint256) {
    (, bool exists) = _getVersionFor(_rollup);
    if (exists) revert Errors.Registry__RollupAlreadyRegistered(_rollup);

    DataStructures.RegistrySnapshot memory newSnapshot =
      DataStructures.RegistrySnapshot(_rollup, block.number);
    currentSnapshot = newSnapshot;
    uint256 version = numberOfVersions++;
    snapshots[version] = newSnapshot;
    rollupToVersion[_rollup] = version;

    return version;
  }

  function _getVersionFor(address _rollup) internal view returns (uint256 version, bool exists) {
    version = rollupToVersion[_rollup];
    return (version, version > 0);
  }
}

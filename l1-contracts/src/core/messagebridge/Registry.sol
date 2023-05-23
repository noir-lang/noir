// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IRegistry} from "@aztec/core/interfaces/messagebridge/IRegistry.sol";
import {IRollup} from "@aztec/core/interfaces/IRollup.sol";
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {IOutbox} from "@aztec/core/interfaces/messagebridge/IOutbox.sol";

// Libraries
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";

/**
 * @title Registry
 * @author Aztec Labs
 * @notice Keeps track of addresses of rollup, inbox and outbox as well as historic addresses.
 * Used as the source of truth for finding the "head" of the rollup chain. Very important information
 * for L1<->L2 communication.
 */
contract Registry is IRegistry {
  uint256 public numberOfVersions;
  DataStructures.RegistrySnapshot internal currentSnapshot;
  mapping(uint256 version => DataStructures.RegistrySnapshot snapshot) internal snapshots;

  /**
   * @notice Creates a new snapshot of the registry
   * todo: this function must be permissioned with some kind of governance/voting/authority
   * @param _rollup - The address of the rollup contract
   * @param _inbox - The address of the inbox contract
   * @param _outbox - The address of the outbox contract
   */
  function upgrade(address _rollup, address _inbox, address _outbox) external override(IRegistry) {
    DataStructures.RegistrySnapshot memory newSnapshot =
      DataStructures.RegistrySnapshot(_rollup, _inbox, _outbox, block.number);
    currentSnapshot = newSnapshot;
    snapshots[numberOfVersions++] = newSnapshot;
  }

  /**
   * @notice Returns the rollup contract
   * @return The rollup contract (of type IRollup)
   */
  function getRollup() external view override(IRegistry) returns (IRollup) {
    return IRollup(currentSnapshot.rollup);
  }

  /**
   * @notice Returns the inbox contract
   * @return The inbox contract (of type IInbox)
   */
  function getInbox() external view override(IRegistry) returns (IInbox) {
    return IInbox(currentSnapshot.inbox);
  }

  /**
   * @notice Returns the outbox contract
   * @return The outbox contract (of type IOutbox)
   */
  function getOutbox() external view override(IRegistry) returns (IOutbox) {
    return IOutbox(currentSnapshot.outbox);
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
}

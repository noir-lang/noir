// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IUnverifiedDataEmitter} from "./interfaces/IUnverifiedDataEmitter.sol";

/**
 * @title UnverifiedDataEmitter
 * @author Aztec Labs
 * @notice Used to log data on chain which are not required to advance the state but are needed for other purposes
 */
contract UnverifiedDataEmitter is IUnverifiedDataEmitter {
  /**
   * @notice Logs data on chain
   * @dev Emits an `UnverifiedData` event
   * @param _l2BlockNum - The l2 block number that the unverified data is related to
   * @param _data - Raw data to share
   */
  function emitUnverifiedData(uint256 _l2BlockNum, bytes calldata _data)
    external
    override(IUnverifiedDataEmitter)
  {
    emit UnverifiedData(_l2BlockNum, msg.sender, _data);
  }

  /**
   * @notice Links L1 and L2 contract addresses
   * @dev Emits a `ContractDeployment` event
   * @dev Unverified and can be emitted by anyone
   * @param _l2BlockNum - The L2 block number that the contract deployment is related to
   * @param _aztecAddress - The address of the L2 counterparty
   * @param _portalAddress - The address of the L1 counterparty
   * @param _acir - The acir bytecode of the L2 contract
   */
  function emitContractDeployment(
    uint256 _l2BlockNum,
    bytes32 _aztecAddress,
    address _portalAddress,
    bytes calldata _acir
  ) external override(IUnverifiedDataEmitter) {
    emit ContractDeployment(_l2BlockNum, _aztecAddress, _portalAddress, _acir);
  }
}

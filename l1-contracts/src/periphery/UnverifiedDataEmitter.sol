// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title UnverifiedDataEmitter
 * @author Aztec Labs
 * @notice Used to log data on chain which are not required to advance the state but are needed for other purposes
 */
contract UnverifiedDataEmitter {
  /**
   * @notice Links L1 and L2 addresses and stores the acir bytecode of the L2 contract
   * @param l2BlockNum - The L2 block number that the information is related to
   * @param aztecAddress - The address of the L2 counterparty
   * @param portalAddress - The address of the L1 counterparty
   * @param acir - The acir bytecode of the L2 contract
   */
  event ContractDeployment(
    uint256 indexed l2BlockNum,
    bytes32 indexed aztecAddress,
    address indexed portalAddress,
    bytes acir
  );

  /**
   * @notice Used to share data which are not required to advance the state but are needed for other purposes
   * @param l2BlockNum - The L2 block number that the information is related to
   * @param sender - The address of the account sharing the information
   * @param data - The information represented as raw bytes
   * @dev Typically contains `TxAuxData` (preimage, contract address and contract slot)
   */
  event UnverifiedData(uint256 indexed l2BlockNum, address indexed sender, bytes data);

  /**
   * @notice Logs data on chain
   * @dev Emits an `UnverifiedData` event
   * @param _l2BlockNum - The l2 block number that the unverified data is related to
   * @param _data - Raw data to share
   */
  function emitUnverifiedData(uint256 _l2BlockNum, bytes calldata _data) external {
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
  ) external {
    emit ContractDeployment(_l2BlockNum, _aztecAddress, _portalAddress, _acir);
  }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title Unverified Data Emitter Interface
 * @author Aztec Labs
 * @notice Interface for Unverified Data Emitter
 * The contract is used to broadcast information that is not part of the rollup block itself
 * and otherwise unverified (thereby the name).
 */
interface IUnverifiedDataEmitter {
  /**
   * @notice Links L1 and L2 addresses and stores the acir bytecode of the L2 contract
   * @param l2BlockNum - The L2 block number that the information is related to
   * @param aztecAddress - The address of the L2 counterparty
   * @param portalAddress - The address of the L1 counterparty
   * @param l2BlockHash - The hash of the L2 block that this is related to
   * @param acir - The acir bytecode of the L2 contract
   */
  event ContractDeployment(
    uint256 indexed l2BlockNum,
    bytes32 indexed aztecAddress,
    address indexed portalAddress,
    bytes32 l2BlockHash,
    bytes acir
  );

  /**
   * @notice Used to share data which are not required to advance the state but are needed for other purposes
   * @param l2BlockNum - The L2 block number that the information is related to
   * @param sender - The address of the account sharing the information
   * @param l2BlockHash - The hash of the L2 block that this is related to.
   * @param data - The information represented as raw bytes
   * @dev Typically contains `TxAuxData` (preimage, contract address and contract slot)
   */
  event UnverifiedData(
    uint256 indexed l2BlockNum, address indexed sender, bytes32 indexed l2BlockHash, bytes data
  );

  function emitUnverifiedData(uint256 _l2BlockNum, bytes32 _l2BlockHash, bytes calldata _data)
    external;

  function emitContractDeployment(
    uint256 _l2BlockNum,
    bytes32 _aztecAddress,
    address _portalAddress,
    bytes32 _l2BlockHash,
    bytes calldata _acir
  ) external;
}

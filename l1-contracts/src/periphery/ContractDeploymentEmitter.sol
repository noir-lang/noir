// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IContractDeploymentEmitter} from "./interfaces/IContractDeploymentEmitter.sol";

/**
 * @title ContractDeploymentEmitter
 * @author Aztec Labs
 * @notice Used to log data on chain which are not required to advance the state but are needed for other purposes
 */
contract ContractDeploymentEmitter is IContractDeploymentEmitter {
  /**
   * @notice Publishes public function bytecode to L1.
   * @dev Emits a `ContractDeployment` event
   * @dev Unverified and can be emitted by anyone
   * @param _l2BlockNum - The L2 block number that the contract deployment is related to
   * @param _aztecAddress - The address of the L2 counterparty
   * @param _portalAddress - The address of the L1 counterparty
   * @param _l2BlockHash - The hash of the L2 block that this is related to
   * @param _partialAddress - The partial address of the deployed contract
   * @param _pubKeyX - The x coordinate of the contract's public key
   * @param _pubKeyY - The y coordinate of the contract's public key
   * @param _acir - The acir bytecode of the L2 contract
   * @dev See the link bellow for more info on partial address and public key:
   * https://github.com/AztecProtocol/aztec-packages/blob/master/docs/docs/concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys
   * TODO: replace the link above with the link to deployed docs
   */
  function emitContractDeployment(
    uint256 _l2BlockNum,
    bytes32 _aztecAddress,
    address _portalAddress,
    bytes32 _l2BlockHash,
    bytes32 _partialAddress,
    bytes32 _pubKeyX,
    bytes32 _pubKeyY,
    bytes calldata _acir
  ) external override(IContractDeploymentEmitter) {
    emit ContractDeployment(
      _l2BlockNum,
      _aztecAddress,
      _portalAddress,
      _l2BlockHash,
      _partialAddress,
      _pubKeyX,
      _pubKeyY,
      _acir
    );
  }
}

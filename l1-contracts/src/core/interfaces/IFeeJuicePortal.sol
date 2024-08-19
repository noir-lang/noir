// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

interface IFeeJuicePortal {
  function initialize(address _registry, address _underlying, bytes32 _l2TokenAddress) external;
  function distributeFees(address _to, uint256 _amount) external;
  function depositToAztecPublic(bytes32 _to, uint256 _amount, bytes32 _secretHash)
    external
    returns (bytes32);
}

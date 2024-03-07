// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

interface IAvailabilityOracle {
  event TxsPublished(bytes32 txsEffectsHash);

  function publish(bytes calldata _body) external returns (bytes32);

  function isAvailable(bytes32 _txsEffectsHash) external view returns (bool);
}

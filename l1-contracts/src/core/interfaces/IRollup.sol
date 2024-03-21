// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

interface IRollup {
  event L2BlockProcessed(uint256 indexed blockNumber);

  function process(bytes calldata _header, bytes32 _archive, bytes memory _proof) external;
}

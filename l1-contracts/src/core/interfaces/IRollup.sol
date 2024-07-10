// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

interface IRollup {
  event L2BlockProcessed(uint256 indexed blockNumber);
  event L2ProofVerified(uint256 indexed blockNumber);

  function process(bytes calldata _header, bytes32 _archive) external;

  function submitProof(
    bytes calldata _header,
    bytes32 _archive,
    bytes calldata _aggregationObject,
    bytes calldata _proof
  ) external;

  function setVerifier(address _verifier) external;
}

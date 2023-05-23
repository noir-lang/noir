// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title Constants Library
 * @author Aztec Labs
 * @notice Library that contains constants used throughout the Aztec protocol
 */
library Constants {
  // Prime field modulus
  uint256 internal constant P =
    21888242871839275222246405745257275088548364400416034343698204186575808495617;

  // Constants used for decoding rollup blocks
  uint256 internal constant COMMITMENTS_PER_KERNEL = 4;
  uint256 internal constant NULLIFIERS_PER_KERNEL = 4;
  uint256 internal constant PUBLIC_DATA_WRITES_PER_KERNEL = 4;
  uint256 internal constant CONTRACTS_PER_KERNEL = 1;
  uint256 internal constant L2_TO_L1_MSGS_PER_KERNEL = 2;
  uint256 internal constant L1_TO_L2_MSGS_PER_ROLLUP = 16;
}

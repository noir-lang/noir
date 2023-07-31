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
  uint256 internal constant MAX_FIELD_VALUE = P - 1;

  // Constants used for decoding rollup blocks
  // TODO(962): Make this constant consistent across the codebase.
  uint256 internal constant COMMITMENTS_PER_TX = 16;
  uint256 internal constant NULLIFIERS_PER_TX = 16;
  uint256 internal constant PUBLIC_DATA_WRITES_PER_TX = 4;
  uint256 internal constant CONTRACTS_PER_TX = 1;
  uint256 internal constant L2_TO_L1_MSGS_PER_TX = 2;
  uint256 internal constant L1_TO_L2_MSGS_PER_BASE_ROLLUP = 16;
  uint256 internal constant KERNELS_PER_BASE_ROLLUP = 2;

  // number of bytes taken up:
  uint256 internal constant COMMITMENTS_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * COMMITMENTS_PER_TX * 0x20;
  uint256 internal constant NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * NULLIFIERS_PER_TX * 0x20;
  uint256 internal constant PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * PUBLIC_DATA_WRITES_PER_TX * 0x40;
  uint256 internal constant CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * CONTRACTS_PER_TX * 0x20;
  uint256 internal constant CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * CONTRACTS_PER_TX * 0x40; //aztec address + eth address (padded to 0x20)
  uint256 internal constant CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED =
    KERNELS_PER_BASE_ROLLUP * CONTRACTS_PER_TX * 0x34; // same as prev except doesn't pad eth address. So 0x20 (aztec address) + 0x14 (eth address)
  uint256 internal constant L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * L2_TO_L1_MSGS_PER_TX * 0x20;
  uint256 internal constant LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP =
    KERNELS_PER_BASE_ROLLUP * 2 * 0x20; // encrypted and unencrypted log types per tx
}

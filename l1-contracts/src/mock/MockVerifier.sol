// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title Mock verifier
 * @author Aztec Labs
 * @notice Will assume that everything is valid proofs
 */
contract MockVerifier {
  function getVerificationKeyHash() public pure returns (bytes32) {
    return bytes32("Im a mock");
  }

  /**
   * @notice A mock verification function that always return true
   * @param _proof - The proof bytes
   * @param _inputs - The public inputs
   * @return True always
   */
  function verify(bytes calldata _proof, bytes32[] calldata _inputs) external view returns (bool) {
    return true;
  }
}

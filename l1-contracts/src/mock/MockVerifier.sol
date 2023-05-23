// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IVerifier} from "./interfaces/IVerifier.sol";

/**
 * @title Mock verifier
 * @author Aztec Labs
 * @notice Will assume that everything is valid proofs
 */
contract MockVerifier is IVerifier {
  /**
   * @notice A mock verification function that always return true
   * @param - The proof bytes, which are ignored
   * @param - The public inputs, which are ignored
   * @return True always
   */
  function verify(bytes calldata, bytes32[] calldata)
    external
    pure
    override(IVerifier)
    returns (bool)
  {
    return true;
  }
}

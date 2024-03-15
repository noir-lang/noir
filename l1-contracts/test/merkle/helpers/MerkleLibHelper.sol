// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {MerkleLib} from "../../../src/core/libraries/MerkleLib.sol";

// A wrapper used to be able to "call" library functions, instead of "jumping" to them, allowing forge to catch the reverts
contract MerkleLibHelper {
  function verifyMembership(
    bytes32[] calldata _path,
    bytes32 _leaf,
    uint256 _index,
    bytes32 _expectedRoot
  ) external pure {
    MerkleLib.verifyMembership(_path, _leaf, _index, _expectedRoot);
  }
}

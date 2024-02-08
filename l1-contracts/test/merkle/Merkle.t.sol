// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

import {NaiveMerkle} from "./Naive.sol";
import {FrontierMerkle} from "./../../src/core/messagebridge/frontier_tree/Frontier.sol";

contract MerkleTest is Test {
  NaiveMerkle internal merkle;
  FrontierMerkle internal frontier;

  uint256 public constant DEPTH = 10;

  function setUp() public {
    merkle = new NaiveMerkle(DEPTH);
    frontier = new FrontierMerkle(DEPTH);
  }

  function testFrontier() public {
    uint256 upper = frontier.SIZE();
    for (uint256 i = 0; i < upper; i++) {
      bytes32 leaf = sha256(abi.encode(i + 1));
      merkle.insertLeaf(leaf);
      frontier.insertLeaf(leaf);
      assertEq(merkle.computeRoot(), frontier.root(), "Frontier Roots should be equal");
    }
  }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

import {NaiveMerkle} from "./Naive.sol";
import {MerkleLibHelper} from "./helpers/MerkleLibHelper.sol";

contract MerkleLibTest is Test {
  MerkleLibHelper internal merkleLibHelper;
  NaiveMerkle internal merkle;
  uint256 public constant DEPTH = 10;

  function setUp() public {
    merkleLibHelper = new MerkleLibHelper();

    merkle = new NaiveMerkle(DEPTH);
    uint256 treeSize = merkle.SIZE();
    for (uint256 i = 0; i < treeSize; i++) {
      bytes32 generatedLeaf = sha256(abi.encode(i + 1));
      merkle.insertLeaf(generatedLeaf);
    }
  }

  function testVerifyMembership(uint256 _idx) public view {
    uint256 leafIndex = bound(_idx, 0, merkle.SIZE() - 1);

    (bytes32[] memory path, bytes32 leaf) = merkle.computeSiblingPath(leafIndex);

    bytes32 expectedRoot = merkle.computeRoot();

    merkleLibHelper.verifyMembership(path, leaf, leafIndex, expectedRoot);
  }

  function testVerifyMembershipWithBadInput(uint256 _idx) public {
    uint256 leafIndex = bound(_idx, 0, merkle.SIZE() - 1);
    bytes32 expectedRoot = merkle.computeRoot();

    // Tests garbled path
    (bytes32[] memory path1, bytes32 leaf) = merkle.computeSiblingPath(leafIndex);
    bytes32 temp1 = path1[0];
    path1[0] = path1[path1.length - 1];
    path1[path1.length - 1] = temp1;
    vm.expectRevert();
    merkleLibHelper.verifyMembership(path1, leaf, leafIndex, expectedRoot);

    // Tests truncated path
    (bytes32[] memory path2,) = merkle.computeSiblingPath(leafIndex);
    bytes32[] memory truncatedPath = new bytes32[](path2.length - 1);
    for (uint256 i = 0; i < truncatedPath.length; i++) {
      truncatedPath[i] = path2[i];
    }

    vm.expectRevert();
    merkleLibHelper.verifyMembership(truncatedPath, leaf, leafIndex, expectedRoot);

    // Tests empty path
    bytes32[] memory emptyPath = new bytes32[](0);
    vm.expectRevert();
    merkleLibHelper.verifyMembership(emptyPath, leaf, leafIndex, expectedRoot);
  }

  function testVerifyMembershipWithRandomSiblingPaths(
    uint256 _idx,
    bytes32[DEPTH] memory _siblingPath
  ) public {
    uint256 leafIndex = _idx % (2 ** DEPTH);
    bytes32 expectedRoot = merkle.computeRoot();

    bytes32[] memory siblingPath = new bytes32[](DEPTH);
    for (uint256 i = 0; i < _siblingPath.length; i++) {
      siblingPath[i] = _siblingPath[i];
    }

    bytes32 leaf = sha256(abi.encode(leafIndex + 1));

    vm.expectRevert();
    merkleLibHelper.verifyMembership(siblingPath, leaf, leafIndex, expectedRoot);
  }
}

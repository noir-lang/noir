// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

import {Hash} from "../../src/core/libraries/Hash.sol";
import {NaiveMerkle} from "./Naive.sol";

contract NaiveTest is Test {
  function setUp() public {}

  function testComputeSiblingPathManuallyLeftChild() public {
    /// Creates a merkle tree with depth 3 and size 8, with leafs from 1 - 8
    NaiveMerkle tree = new NaiveMerkle(3);
    for (uint256 i = 1; i <= 8; i++) {
      bytes32 generatedLeaf = bytes32(abi.encode(i));
      tree.insertLeaf(generatedLeaf);
    }

    /**
     * We manually make a path; this is the sibling path of the leaf with the value of 1.
     * This path, from leaf to root, consists a, b, and c; which correspond to the value of 2, then the hash of 3 and 4,
     * and finally, the hash of 5 and 6 concatenated with the hash of 7 and 8;
     * d0:                                            [ root ]
     * d1:                      [ ]                                               [c]
     * d2:         [ ]                      [b]                       [ ]                     [ ]
     * d3:   [1]         [a]          [3]         [4]           [5]         [6]          [7]        [8].
     */
    bytes32[3] memory expectedPath = [
      bytes32(abi.encode(2)),
      Hash.sha256ToField(bytes.concat(bytes32(abi.encode(3)), bytes32(abi.encode(4)))),
      Hash.sha256ToField(
        bytes.concat(
          Hash.sha256ToField(bytes.concat(bytes32(abi.encode(5)), bytes32(abi.encode(6)))),
          Hash.sha256ToField(bytes.concat(bytes32(abi.encode(7)), bytes32(abi.encode(8))))
        )
      )
    ];

    /// We then compute the sibling path using the tree and expect that our manual calculation should equal the computed one
    (bytes32[] memory path, bytes32 leaf) = tree.computeSiblingPath(0);
    assertEq(leaf, bytes32(abi.encode(1)));
    assertEq(path[0], expectedPath[0]);
    assertEq(path[1], expectedPath[1]);
    assertEq(path[2], expectedPath[2]);
  }

  function testComputeSiblingPathManuallyRightChild() public {
    /// Creates a merkle tree with depth 3 and size 8, with leafs from 1 - 8
    NaiveMerkle tree = new NaiveMerkle(3);
    for (uint256 i = 1; i <= 8; i++) {
      bytes32 generatedLeaf = bytes32(abi.encode(i));
      tree.insertLeaf(generatedLeaf);
    }

    /**
     * We manually make a path; this is the sibling path of the leaf with the value of 8.
     * This path, from leaf to root, consists of c a, b, and c; which correspond to the value of 7, then the hash of 5 and 6,
     * and finally, the hash of 1 and 2 concatenated with the hash of 3 and 4;
     * d0:                                            [ root ]
     * d1:                      [c]                                               [ ]
     * d2:         [ ]                      [b]                       [b]                     [ ]
     * d3:   [1]         [2]          [3]         [4]           [5]         [6]          [a]        [8].
     */
    bytes32[3] memory expectedPath = [
      bytes32(abi.encode(7)),
      Hash.sha256ToField(bytes.concat(bytes32(abi.encode(5)), bytes32(abi.encode(6)))),
      Hash.sha256ToField(
        bytes.concat(
          Hash.sha256ToField(bytes.concat(bytes32(abi.encode(1)), bytes32(abi.encode(2)))),
          Hash.sha256ToField(bytes.concat(bytes32(abi.encode(3)), bytes32(abi.encode(4))))
        )
      )
    ];

    /// We then compute the sibling path using the tree and expect that our manual calculation should equal the computed one
    (bytes32[] memory path, bytes32 leaf) = tree.computeSiblingPath(7);
    assertEq(leaf, bytes32(abi.encode(8)));
    assertEq(path[0], expectedPath[0]);
    assertEq(path[1], expectedPath[1]);
    assertEq(path[2], expectedPath[2]);
  }
}

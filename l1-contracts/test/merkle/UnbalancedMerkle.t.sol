// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";
import {Hash} from "../../src/core/libraries/Hash.sol";

import {TxsDecoderHelper} from "../decoders/helpers/TxsDecoderHelper.sol";
/**
 * Tests the tree construction for unbalanced rollups.
 * Used for calculating txsEffectsHash over non balanced rollups - each leaf is one baseLeaf
 * calculated in TxsDecoder.sol.
 */

contract UnbalancedMerkleTest is Test {
  /**
   * Rollups are constructed greedily, with a set of N transactions split into subtrees of decreasing
   * powers of 2.
   * We list them in reverse order as we compute subtree roots from R to L
   */
  TxsDecoderHelper internal txsHelper;

  function setUp() public {
    txsHelper = new TxsDecoderHelper();
  }

  function testDecomp() public {
    // Worst case - max num txs
    uint32 numTxs = 65535;
    (uint256 min, uint256 max) = txsHelper.computeMinMaxPathLength(numTxs);
    assertEq(min, 15);
    assertEq(max, 16);
    // Single tree of 2**15
    numTxs = 32768;
    (min, max) = txsHelper.computeMinMaxPathLength(numTxs);
    assertEq(min, 15);
    assertEq(max, 15);
    // Single tree of 2**13
    numTxs = 8192;
    (min, max) = txsHelper.computeMinMaxPathLength(numTxs);
    assertEq(min, 13);
    assertEq(max, 13);
    // Trees of 2**12, 2**11, ... 2**0
    numTxs = 8191;
    (min, max) = txsHelper.computeMinMaxPathLength(numTxs);
    assertEq(min, 12);
    assertEq(max, 13);
    // Single tree of 2**8
    numTxs = 256;
    (min, max) = txsHelper.computeMinMaxPathLength(numTxs);
    assertEq(min, 8);
    assertEq(max, 8);
    // Left subtree of 2**8, right subtree of single leaf
    numTxs = 257;
    (min, max) = txsHelper.computeMinMaxPathLength(numTxs);
    assertEq(min, 1);
    assertEq(max, 9);
  }

  // Example - 2 txs:
  //
  //   root
  //  /     \
  // base  base
  function testComputeTxsEffectsHash2() public {
    // Generate some base leaves
    bytes32[] memory baseLeaves = new bytes32[](2);
    for (uint256 i = 0; i < 2; i++) {
      baseLeaves[i] = Hash.sha256ToField(abi.encodePacked(i));
    }
    // We have just one 'balanced' branch, so depth is 0 with 2 elements
    (uint256 min, uint256 max) = txsHelper.computeMinMaxPathLength(2);
    assertEq(min, 1);
    assertEq(max, 1);
    bytes32 rootTxsEffectsHash = Hash.sha256ToField(bytes.concat(baseLeaves[0], baseLeaves[1]));
    bytes32 calculatedTxsEffectsHash = txsHelper.computeUnbalancedRoot(baseLeaves);
    assertEq(calculatedTxsEffectsHash, rootTxsEffectsHash);
  }
  // Example - 3 txs:
  //
  //        root
  //     /        \
  //   merge     base
  //  /     \
  // base  base

  function testComputeTxsEffectsHash3() public {
    // Generate some base leaves
    bytes32[] memory baseLeaves = new bytes32[](3);
    for (uint256 i = 0; i < 3; i++) {
      baseLeaves[i] = Hash.sha256ToField(abi.encodePacked(i));
    }
    (uint256 min, uint256 max) = txsHelper.computeMinMaxPathLength(3);
    assertEq(min, 1);
    assertEq(max, 2);
    bytes32 mergeTxsEffectsHash = Hash.sha256ToField(bytes.concat(baseLeaves[0], baseLeaves[1]));
    bytes32 rootTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(mergeTxsEffectsHash, baseLeaves[2]));
    bytes32 calculatedTxsEffectsHash = txsHelper.computeUnbalancedRoot(baseLeaves);
    assertEq(calculatedTxsEffectsHash, rootTxsEffectsHash);
  }

  // Example - 5 txs:
  //
  //                  root
  //             /            \
  //          merge           base
  //      /          \
  //   merge        merge
  //  /     \      /     \
  // base  base  base   base
  function testComputeTxsEffectsHash5() public {
    // Generate some base leaves
    bytes32[] memory baseLeaves = new bytes32[](5);
    for (uint256 i = 0; i < 5; i++) {
      baseLeaves[i] = Hash.sha256ToField(abi.encodePacked(i));
    }
    (uint256 min, uint256 max) = txsHelper.computeMinMaxPathLength(5);
    assertEq(min, 1);
    assertEq(max, 3);
    bytes32 firstMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(baseLeaves[0], baseLeaves[1]));
    bytes32 secondMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(baseLeaves[2], baseLeaves[3]));
    bytes32 thirdMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(firstMergeTxsEffectsHash, secondMergeTxsEffectsHash));
    bytes32 rootTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(thirdMergeTxsEffectsHash, baseLeaves[4]));
    bytes32 calculatedTxsEffectsHash = txsHelper.computeUnbalancedRoot(baseLeaves);
    assertEq(calculatedTxsEffectsHash, rootTxsEffectsHash);
  }

  // Example - 6 txs:
  //
  //                  root
  //             /            \
  //         merge4           merge3
  //      /          \        /    \
  //   merge1       merge2  base  base
  //  /     \      /     \
  // base  base  base   base
  function testComputeTxsEffectsHash6() public {
    // Generate some base leaves
    bytes32[] memory baseLeaves = new bytes32[](6);
    for (uint256 i = 0; i < 6; i++) {
      baseLeaves[i] = Hash.sha256ToField(abi.encodePacked(i));
    }
    (uint256 min, uint256 max) = txsHelper.computeMinMaxPathLength(6);
    assertEq(min, 2);
    assertEq(max, 3);
    bytes32 firstMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(baseLeaves[0], baseLeaves[1]));
    bytes32 secondMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(baseLeaves[2], baseLeaves[3]));
    bytes32 thirdMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(baseLeaves[4], baseLeaves[5]));
    bytes32 fourthMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(firstMergeTxsEffectsHash, secondMergeTxsEffectsHash));
    bytes32 rootTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(fourthMergeTxsEffectsHash, thirdMergeTxsEffectsHash));
    bytes32 calculatedTxsEffectsHash = txsHelper.computeUnbalancedRoot(baseLeaves);
    assertEq(calculatedTxsEffectsHash, rootTxsEffectsHash);
  }

  // Example - 7 txs:
  //
  //                     root
  //             /                  \
  //         merge3                merge5
  //      /          \             /    \
  //   merge1       merge2      merge4  base
  //  /     \      /     \      /    \
  // base  base  base   base  base  base
  function testComputeTxsEffectsHash7() public {
    // Generate some base leaves
    bytes32[] memory baseLeaves = new bytes32[](7);
    for (uint256 i = 0; i < 6; i++) {
      baseLeaves[i] = Hash.sha256ToField(abi.encodePacked(i));
    }
    (uint256 min, uint256 max) = txsHelper.computeMinMaxPathLength(7);
    assertEq(min, 2);
    assertEq(max, 3);
    bytes32 firstMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(baseLeaves[0], baseLeaves[1]));
    bytes32 secondMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(baseLeaves[2], baseLeaves[3]));
    bytes32 thirdMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(firstMergeTxsEffectsHash, secondMergeTxsEffectsHash));
    bytes32 fourthMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(baseLeaves[4], baseLeaves[5]));
    bytes32 fifthMergeTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(fourthMergeTxsEffectsHash, baseLeaves[6]));

    bytes32 rootTxsEffectsHash =
      Hash.sha256ToField(bytes.concat(thirdMergeTxsEffectsHash, fifthMergeTxsEffectsHash));
    bytes32 calculatedTxsEffectsHash = txsHelper.computeUnbalancedRoot(baseLeaves);
    assertEq(calculatedTxsEffectsHash, rootTxsEffectsHash);
  }
}

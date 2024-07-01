// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Errors} from "../libraries/Errors.sol";
import {Hash} from "../libraries/Hash.sol";

/**
 * @title Merkle Library
 * @author Aztec Labs
 * @notice Library that contains functions useful when interacting with Merkle Trees
 */
library MerkleLib {
  /**
   * @notice Verifies the membership of a leaf and path against an expected root.
   * @dev In the case of a mismatched root, and subsequent inability to verify membership, this function throws.
   * @param _path - The sibling path of the message as a leaf, used to prove message inclusion
   * @param _leaf - The hash of the message we are trying to prove inclusion for
   * @param _index - The index of the message inside the L2 to L1 message tree
   * @param _expectedRoot - The expected root to check the validity of the message and sibling path with.
   * @notice -
   * E.g. A sibling path for a leaf at index 3 (L) in a tree of depth 3 (between 5 and 8 leafs) consists of the 3 elements denoted as *'s
   * d0:                                            [ root ]
   * d1:                      [ ]                                               [*]
   * d2:         [*]                      [ ]                       [ ]                     [ ]
   * d3:   [ ]         [ ]          [*]         [L]           [ ]         [ ]          [ ]        [ ].
   * And the elements would be ordered as: [ d3_index_2, d2_index_0, d1_index_1 ].
   */
  function verifyMembership(
    bytes32[] calldata _path,
    bytes32 _leaf,
    uint256 _index,
    bytes32 _expectedRoot
  ) internal pure {
    bytes32 subtreeRoot = _leaf;
    /// @notice - We use the indexAtHeight to see whether our child of the next subtree is at the left or the right side
    uint256 indexAtHeight = _index;

    for (uint256 height = 0; height < _path.length; height++) {
      /// @notice - This affects the way we concatenate our two children to then hash and calculate the root, as any odd indexes (index bit-masked with least significant bit) are right-sided children.
      bool isRight = (indexAtHeight & 1) == 1;

      subtreeRoot = isRight
        ? Hash.sha256ToField(bytes.concat(_path[height], subtreeRoot))
        : Hash.sha256ToField(bytes.concat(subtreeRoot, _path[height]));
      /// @notice - We divide by two here to get the index of the parent of the current subtreeRoot in its own layer
      indexAtHeight >>= 1;
    }

    if (subtreeRoot != _expectedRoot) {
      revert Errors.MerkleLib__InvalidRoot(_expectedRoot, subtreeRoot, _leaf, _index);
    }
  }

  /**
   * @notice Computes the minimum and maximum path size of an unbalanced tree.
   * @dev Follows structure of rollup circuits by greedy filling subtrees.
   * @param _numTxs - The number of txs to form into subtrees.
   * @return (min, max) - The min and max path sizes.
   */
  function computeMinMaxPathLength(uint256 _numTxs) internal pure returns (uint256, uint256) {
    uint256 numTxs = _numTxs < 2 ? 2 : _numTxs;
    uint256 numSubtrees = 0;
    uint256 currentSubtreeSize = 1;
    uint256 currentSubtreeHeight = 0;
    uint256 firstSubtreeHeight;
    uint256 finalSubtreeHeight;
    while (numTxs != 0) {
      // If size & txs == 0, the subtree doesn't exist for this number of txs
      if (currentSubtreeSize & numTxs == 0) {
        currentSubtreeSize <<= 1;
        currentSubtreeHeight++;
        continue;
      }
      // Assign the smallest rightmost subtree height
      if (numSubtrees == 0) finalSubtreeHeight = currentSubtreeHeight;
      // Assign the largest leftmost subtree height
      if (numTxs - currentSubtreeSize == 0) firstSubtreeHeight = currentSubtreeHeight;
      numTxs -= currentSubtreeSize;
      currentSubtreeSize <<= 1;
      currentSubtreeHeight++;
      numSubtrees++;
    }
    if (numSubtrees == 1) {
      // We have a balanced tree
      return (firstSubtreeHeight, firstSubtreeHeight);
    }
    uint256 min = finalSubtreeHeight + numSubtrees - 1;
    uint256 max = firstSubtreeHeight + 1;
    return (min, max);
  }

  /**
   * @notice Calculates a tree height from the amount of elements in the tree
   * @dev - This mirrors the function in TestUtil, but assumes _size is an exact power of 2 or = 1
   * @param _size - The number of elements in the tree
   */
  function calculateTreeHeightFromSize(uint256 _size) internal pure returns (uint256) {
    /// We need the height of the tree that will contain all of our leaves,
    /// hence the next highest power of two from the amount of leaves - Math.ceil(Math.log2(x))
    uint256 height = 0;

    if (_size == 1) {
      return 0;
    }

    /// While size > 1, we divide by two, and count how many times we do this; producing a rudimentary way of calculating Math.Floor(Math.log2(x))
    while (_size > 1) {
      _size >>= 1;
      height++;
    }
    return height;
  }
}

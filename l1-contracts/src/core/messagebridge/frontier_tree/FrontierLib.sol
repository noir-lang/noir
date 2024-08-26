// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Hash} from "../../libraries/Hash.sol";

/**
 * @title FrontierLib
 * @author Aztec Labs
 * @notice Library for managing frontier trees.
 */
library FrontierLib {
  struct Forest {
    mapping(uint256 index => bytes32 zero) zeros;
  }

  struct Tree {
    uint256 nextIndex;
    mapping(uint256 => bytes32) frontier;
  }

  function initialize(Forest storage self, uint256 _height) internal {
    self.zeros[0] = bytes32(0);
    for (uint256 i = 1; i <= _height; i++) {
      self.zeros[i] = Hash.sha256ToField(bytes.concat(self.zeros[i - 1], self.zeros[i - 1]));
    }
  }

  function insertLeaf(Tree storage self, bytes32 _leaf) internal returns (uint256) {
    uint256 index = self.nextIndex;
    uint256 level = _computeLevel(index);
    bytes32 right = _leaf;
    for (uint256 i = 0; i < level; i++) {
      right = Hash.sha256ToField(bytes.concat(self.frontier[i], right));
    }
    self.frontier[level] = right;

    self.nextIndex++;

    return index;
  }

  function root(Tree storage self, Forest storage _forest, uint256 _height, uint256 _size)
    internal
    view
    returns (bytes32)
  {
    uint256 next = self.nextIndex;
    if (next == 0) {
      return _forest.zeros[_height];
    }
    if (next == _size) {
      return self.frontier[_height];
    }

    uint256 index = next - 1;
    uint256 level = _computeLevel(index);

    // We should start at the highest frontier level with a left leaf
    bytes32 temp = self.frontier[level];

    uint256 bits = index >> level;
    for (uint256 i = level; i < _height; i++) {
      bool isRight = bits & 1 == 1;
      if (isRight) {
        if (self.frontier[i] == temp) {
          // We will never hit the case that frontier[i] == temp
          // because this require that frontier[i] is the right child
          // and in that case we started higher up the tree
          revert("Mistakes were made");
        }
        temp = Hash.sha256ToField(bytes.concat(self.frontier[i], temp));
      } else {
        temp = Hash.sha256ToField(bytes.concat(temp, _forest.zeros[i]));
      }
      bits >>= 1;
    }

    return temp;
  }

  function isFull(Tree storage self, uint256 _size) internal view returns (bool) {
    return self.nextIndex == _size;
  }

  function _computeLevel(uint256 _leafIndex) internal pure returns (uint256) {
    // The number of trailing ones is how many times in a row we are the right child.
    // e.g., each time this happens we go another layer up to update the parent.
    uint256 count = 0;
    uint256 index = _leafIndex;
    while (index & 1 == 1) {
      count++;
      index >>= 1;
    }
    return count;
  }
}

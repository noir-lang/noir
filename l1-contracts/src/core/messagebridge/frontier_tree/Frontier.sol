// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Hash} from "../../libraries/Hash.sol";
import {IFrontier} from "../../interfaces/messagebridge/IFrontier.sol";

// This truncates each hash and hash preimage to 31 bytes to follow Noir.
// It follows the logic in /noir-protocol-circuits/crates/parity-lib/src/utils/sha256_merkle_tree.nr
// TODO(Miranda): Possibly nuke this contract, and use a generic version which can either use
// regular sha256 or sha256ToField when emulating circuits
contract FrontierMerkle is IFrontier {
  uint256 public immutable HEIGHT;
  uint256 public immutable SIZE;

  uint256 internal nextIndex = 0;

  mapping(uint256 level => bytes32 node) public frontier;

  // Below can be pre-computed so it would be possible to have constants
  // for the zeros at each level. This would save gas on computations
  mapping(uint256 level => bytes32 zero) public zeros;

  constructor(uint256 _height) {
    HEIGHT = _height;
    SIZE = 2 ** _height;

    zeros[0] = bytes32(0);
    for (uint256 i = 1; i <= HEIGHT; i++) {
      zeros[i] = Hash.sha256ToField(bytes.concat(zeros[i - 1], zeros[i - 1]));
    }
  }

  function insertLeaf(bytes32 _leaf) external override(IFrontier) returns (uint256) {
    uint256 index = nextIndex;
    uint256 level = _computeLevel(index);
    bytes32 right = _leaf;
    for (uint256 i = 0; i < level; i++) {
      right = Hash.sha256ToField(bytes.concat(frontier[i], right));
    }
    frontier[level] = right;

    nextIndex++;

    return index;
  }

  function root() external view override(IFrontier) returns (bytes32) {
    uint256 next = nextIndex;
    if (next == 0) {
      return zeros[HEIGHT];
    }
    if (next == SIZE) {
      return frontier[HEIGHT];
    }

    uint256 index = next - 1;
    uint256 level = _computeLevel(index);

    // We should start at the highest frontier level with a left leaf
    bytes32 temp = frontier[level];

    uint256 bits = index >> level;
    for (uint256 i = level; i < HEIGHT; i++) {
      bool isRight = bits & 1 == 1;
      if (isRight) {
        if (frontier[i] == temp) {
          // We will never hit the case that frontier[i] == temp
          // because this require that frontier[i] is the right child
          // and in that case we started higher up the tree
          revert("Mistakes were made");
        }
        temp = Hash.sha256ToField(bytes.concat(frontier[i], temp));
      } else {
        temp = Hash.sha256ToField(bytes.concat(temp, zeros[i]));
      }
      bits >>= 1;
    }

    return temp;
  }

  function isFull() external view override(IFrontier) returns (bool) {
    return nextIndex == SIZE;
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

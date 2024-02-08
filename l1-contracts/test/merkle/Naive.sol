// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

contract NaiveMerkle {
  uint256 public immutable DEPTH;
  uint256 public immutable SIZE;

  uint256 public nextIndex = 0;

  mapping(uint256 index => bytes32 leaf) public leafs;

  constructor(uint256 _depth) {
    DEPTH = _depth;
    SIZE = 2 ** _depth;
  }

  function insertLeaf(bytes32 _leaf) public {
    leafs[nextIndex++] = _leaf;
  }

  function computeRoot() public view returns (bytes32) {
    bytes32[] memory nodes = new bytes32[](SIZE / 2);
    uint256 size = SIZE;
    for (uint256 i = 0; i < DEPTH; i++) {
      for (uint256 j = 0; j < size; j += 2) {
        if (i == 0) {
          nodes[j / 2] = sha256(bytes.concat(leafs[j], leafs[j + 1]));
        } else {
          nodes[j / 2] = sha256(bytes.concat(nodes[j], nodes[j + 1]));
        }
      }
      size /= 2;
    }
    return nodes[0];
  }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Hash} from "../../src/core/libraries/Hash.sol";

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
          nodes[j / 2] = Hash.sha256ToField(bytes.concat(leafs[j], leafs[j + 1]));
        } else {
          nodes[j / 2] = Hash.sha256ToField(bytes.concat(nodes[j], nodes[j + 1]));
        }
      }
      size /= 2;
    }
    return nodes[0];
  }

  function computeSiblingPath(uint256 _index) public view returns (bytes32[] memory, bytes32) {
    bytes32[] memory nodes = new bytes32[](SIZE / 2);
    bytes32[] memory path = new bytes32[](DEPTH);

    uint256 idx = _index;

    uint256 size = SIZE;
    for (uint256 i = 0; i < DEPTH; i++) {
      bool isRight = (idx & 1) == 1;
      if (i > 0) {
        path[i] = isRight ? nodes[idx - 1] : nodes[idx + 1];
      } else {
        path[i] = isRight ? leafs[idx - 1] : leafs[idx + 1];
      }

      for (uint256 j = 0; j < size; j += 2) {
        if (i == 0) {
          nodes[j / 2] = Hash.sha256ToField(bytes.concat(leafs[j], leafs[j + 1]));
        } else {
          nodes[j / 2] = Hash.sha256ToField(bytes.concat(nodes[j], nodes[j + 1]));
        }
      }

      idx /= 2;
      size /= 2;
    }

    return (path, leafs[_index]);
  }
}

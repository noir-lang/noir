// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Inbox} from "../../src/core/messagebridge/Inbox.sol";
import {FrontierLib} from "../../src/core/messagebridge/frontier_tree/FrontierLib.sol";

// Libraries
import {Constants} from "../../src/core/libraries/ConstantsGen.sol";

contract InboxHarness is Inbox {
  using FrontierLib for FrontierLib.Tree;

  constructor(address _rollup, uint256 _height) Inbox(_rollup, _height) {}

  function getSize() external view returns (uint256) {
    return SIZE;
  }

  function getEmptyRoot() external view returns (bytes32) {
    return EMPTY_ROOT;
  }

  function treeInProgressFull() external view returns (bool) {
    return trees[inProgress].isFull(SIZE);
  }

  function getToConsumeRoot(uint256 _toConsume) external view returns (bytes32) {
    bytes32 root = EMPTY_ROOT;
    if (_toConsume > Constants.INITIAL_L2_BLOCK_NUM) {
      root = trees[_toConsume].root(forest, HEIGHT, SIZE);
    }
    return root;
  }

  function getNumTrees() external view returns (uint256) {
    // -INITIAL_L2_BLOCK_NUM because tree number INITIAL_L2_BLOCK_NUM is not real
    return inProgress - Constants.INITIAL_L2_BLOCK_NUM;
  }
}

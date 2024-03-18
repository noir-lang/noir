// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Inbox} from "../../src/core/messagebridge/Inbox.sol";

// Libraries
import {Constants} from "../../src/core/libraries/ConstantsGen.sol";

contract InboxHarness is Inbox {
  constructor(address _rollup, uint256 _height) Inbox(_rollup, _height) {}

  function getSize() external view returns (uint256) {
    return SIZE;
  }

  function getEmptyRoot() external view returns (bytes32) {
    return EMPTY_ROOT;
  }

  function treeInProgressFull() external view returns (bool) {
    return trees[inProgress].isFull();
  }

  function getToConsumeRoot() external view returns (bytes32) {
    bytes32 root = EMPTY_ROOT;
    if (toConsume > Constants.INITIAL_L2_BLOCK_NUM) {
      root = trees[toConsume].root();
    }
    return root;
  }

  function getNumTrees() external view returns (uint256) {
    // -INITIAL_L2_BLOCK_NUM because tree number INITIAL_L2_BLOCK_NUM is not real
    return inProgress - Constants.INITIAL_L2_BLOCK_NUM;
  }
}

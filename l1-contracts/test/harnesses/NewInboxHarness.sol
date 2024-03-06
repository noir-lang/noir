// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {NewInbox} from "../../src/core/messagebridge/NewInbox.sol";

// Libraries
import {Constants} from "../../src/core/libraries/ConstantsGen.sol";

// TODO: rename to InboxHarness once all the pieces of the new message model are in place.
contract NewInboxHarness is NewInbox {
  constructor(address _rollup, uint256 _height) NewInbox(_rollup, _height) {}

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

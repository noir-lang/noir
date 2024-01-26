// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {HeaderLib} from "../../../src/core/libraries/HeaderLib.sol";

contract HeaderLibHelper {
  // A wrapper used such that we get "calldata" and not memory
  function decode(bytes calldata _header) public pure returns (HeaderLib.Header memory) {
    return HeaderLib.decode(_header);
  }
}

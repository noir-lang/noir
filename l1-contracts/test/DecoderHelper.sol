// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.18;

import {Decoder} from "@aztec3/core/Decoder.sol";
import {Rollup} from "@aztec3/core/Rollup.sol";

contract DecoderHelper is Decoder {
  function decode(bytes calldata _l2Block)
    external
    pure
    returns (uint256, bytes32, bytes32, bytes32)
  {
    return _decode(_l2Block);
  }

  function computeDiffRoot(bytes calldata _l2Block) external pure returns (bytes32) {
    return _computeDiffRoot(_l2Block);
  }
}

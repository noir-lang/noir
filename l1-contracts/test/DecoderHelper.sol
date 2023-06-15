// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Decoder} from "@aztec/core/libraries/Decoder.sol";
import {Rollup} from "@aztec/core/Rollup.sol";

contract DecoderHelper {
  function decode(bytes calldata _l2Block)
    external
    pure
    returns (uint256, bytes32, bytes32, bytes32, bytes32[] memory, bytes32[] memory)
  {
    return Decoder.decode(_l2Block);
  }

  function computeDiffRootAndMessagesHash(bytes calldata _l2Block)
    external
    pure
    returns (bytes32, bytes32)
  {
    (bytes32 diffRoot, bytes32 l1ToL2MessagesHash,,) = Decoder.computeConsumables(_l2Block);
    return (diffRoot, l1ToL2MessagesHash);
  }

  function computeKernelLogsHash(bytes calldata _kernelLogs)
    external
    pure
    returns (bytes32, uint256)
  {
    (bytes32 logsHash, uint256 offset) = Decoder.computeKernelLogsHash(0, _kernelLogs);

    return (logsHash, offset);
  }
}

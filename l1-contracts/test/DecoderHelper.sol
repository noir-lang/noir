// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Decoder} from "@aztec/core/Decoder.sol";
import {Rollup} from "@aztec/core/Rollup.sol";

contract DecoderHelper is Decoder {
  function decode(bytes calldata _l2Block)
    external
    pure
    returns (uint256, bytes32, bytes32, bytes32, bytes32[] memory, bytes32[] memory)
  {
    return _decode(_l2Block);
  }

  function computeDiffRootAndMessagesHash(bytes calldata _l2Block)
    external
    pure
    returns (bytes32, bytes32)
  {
    (bytes32 diffRoot, bytes32 l1ToL2MessagesHash,,) = _computeConsumables(_l2Block);
    return (diffRoot, l1ToL2MessagesHash);
  }

  function computeKernelLogsHash(bytes calldata _kernelLogs)
    external
    pure
    returns (bytes32, uint256)
  {
    uint256 offsetInCalldata;
    assembly {
      offsetInCalldata := _kernelLogs.offset
    }

    (bytes32 logsHash, uint256 offset) = _computeKernelLogsHash(offsetInCalldata, _kernelLogs);
    uint256 bytesAdvanced = offset - offsetInCalldata;

    return (logsHash, bytesAdvanced);
  }
}

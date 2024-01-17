// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Decoder} from "../../../src/core/libraries/decoders/Decoder.sol";
import {Rollup} from "../../../src/core/Rollup.sol";

contract DecoderHelper {
  function computeConsumables(bytes calldata _body)
    external
    pure
    returns (bytes32, bytes32, bytes32[] memory, bytes32[] memory)
  {
    return Decoder.computeConsumables(_body);
  }

  function computeKernelLogsHash(bytes calldata _kernelLogs)
    external
    pure
    returns (bytes32, uint256)
  {
    return Decoder.computeKernelLogsHash(0, _kernelLogs);
  }
}

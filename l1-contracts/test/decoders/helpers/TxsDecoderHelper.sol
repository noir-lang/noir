// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {TxsDecoder} from "../../../src/core/libraries/decoders/TxsDecoder.sol";

contract TxsDecoderHelper {
  // A wrapper used such that we get "calldata" and not memory
  function decode(bytes calldata _body) public pure returns (bytes32 txsHash) {
    return TxsDecoder.decode(_body);
  }

  function computeKernelLogsHash(bytes calldata _kernelLogs)
    external
    pure
    returns (bytes32, uint256)
  {
    return TxsDecoder.computeKernelLogsHash(0, _kernelLogs);
  }

  function computeNumTxEffectsToPad(uint32 _numTxEffects) external pure returns (uint32) {
    return TxsDecoder.computeNumTxEffectsToPad(_numTxEffects);
  }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {TxsDecoder} from "../../../src/core/libraries/decoders/TxsDecoder.sol";
import {MerkleLib} from "../../../src/core/libraries/MerkleLib.sol";

contract TxsDecoderHelper {
  // A wrapper used such that we get "calldata" and not memory
  function decode(bytes calldata _body) public pure returns (bytes32 txsHash) {
    return TxsDecoder.decode(_body);
  }

  function computeKernelLogsHash(bytes calldata _kernelLogs)
    external
    pure
    returns (bytes32, uint256, uint256)
  {
    return TxsDecoder.computeKernelEncryptedLogsHash(0, _kernelLogs);
  }

  function computeTxOutHash(bytes calldata _kernelMsgs) external pure returns (bytes32) {
    return TxsDecoder.computeTxOutHash(1, _kernelMsgs);
  }

  function computeNumTxEffectsToPad(uint32 _numTxEffects) external pure returns (uint32) {
    return TxsDecoder.computeNumTxEffectsToPad(_numTxEffects);
  }

  function computeNumMsgsToPad(uint32 _numL2toL1Msgs) external pure returns (uint32) {
    return TxsDecoder.computeNumMsgsToPad(_numL2toL1Msgs);
  }

  function computeUnbalancedRoot(bytes32[] memory _leaves) external pure returns (bytes32) {
    return TxsDecoder.computeUnbalancedRoot(_leaves);
  }

  function computeMinMaxPathLength(uint32 _numTxEffects) external pure returns (uint256, uint256) {
    return MerkleLib.computeMinMaxPathLength(_numTxEffects);
  }
}

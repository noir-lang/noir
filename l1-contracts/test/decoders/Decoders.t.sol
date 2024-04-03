// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DecoderBase} from "./Base.sol";

import {Hash} from "../../src/core/libraries/Hash.sol";
import {DataStructures} from "../../src/core/libraries/DataStructures.sol";

import {HeaderLibHelper} from "./helpers/HeaderLibHelper.sol";
import {TxsDecoderHelper} from "./helpers/TxsDecoderHelper.sol";
import {HeaderLib} from "../../src/core/libraries/HeaderLib.sol";

import {TxsDecoder} from "../../src/core/libraries/decoders/TxsDecoder.sol";

import {AvailabilityOracle} from "../../src/core/availability_oracle/AvailabilityOracle.sol";

/**
 * Blocks are generated using the `integration_l1_publisher.test.ts` tests.
 * Main use of these test is shorter cycles when updating the decoder contract.
 * All tests here are skipped (all tests are prefixed with an underscore)!
 * This is because we implicitly test the decoding in integration_l1_publisher.test.ts
 */
contract DecodersTest is DecoderBase {
  HeaderLibHelper internal headerHelper;
  TxsDecoderHelper internal txsHelper;

  function setUp() public virtual {
    headerHelper = new HeaderLibHelper();
    txsHelper = new TxsDecoderHelper();
  }

  function testDecodeBlocks() public {
    _testDecodeBlock("mixed_block_0");
    _testDecodeBlock("mixed_block_1");
    _testDecodeBlock("empty_block_0");
    _testDecodeBlock("empty_block_1");
  }

  function _testDecodeBlock(string memory name) public virtual {
    DecoderBase.Full memory data = load(name);

    // Header
    {
      DecoderBase.DecodedHeader memory referenceHeader = data.block.decodedHeader;
      HeaderLib.Header memory header = headerHelper.decode(data.block.header);

      // GlobalVariables
      {
        DecoderBase.GlobalVariables memory globalVariables = referenceHeader.globalVariables;

        assertEq(
          header.globalVariables.blockNumber, globalVariables.blockNumber, "Invalid block number"
        );
        assertEq(header.globalVariables.chainId, globalVariables.chainId, "Invalid chain Id");
        assertEq(header.globalVariables.timestamp, globalVariables.timestamp, "Invalid timestamp");
        assertEq(header.globalVariables.version, globalVariables.version, "Invalid version");
        assertEq(header.globalVariables.coinbase, globalVariables.coinbase, "Invalid coinbase");
        assertEq(
          header.globalVariables.feeRecipient, globalVariables.feeRecipient, "Invalid feeRecipient"
        );
      }

      // ContentCommitment
      {
        DecoderBase.ContentCommitment memory contentCommitment = referenceHeader.contentCommitment;

        assertEq(
          header.contentCommitment.txTreeHeight,
          contentCommitment.txTreeHeight,
          "Invalid txTreeSize"
        );
        assertEq(
          header.contentCommitment.txsEffectsHash,
          contentCommitment.txsEffectsHash,
          "Invalid txsEffectsHash"
        );
        assertEq(header.contentCommitment.inHash, contentCommitment.inHash, "Invalid inHash");
        assertEq(header.contentCommitment.outHash, contentCommitment.outHash, "Invalid outHash");
      }

      // StateReference
      {
        DecoderBase.StateReference memory stateReference = referenceHeader.stateReference;

        // L1 -> L2 messages
        assertEq(
          header.stateReference.l1ToL2MessageTree.nextAvailableLeafIndex,
          stateReference.l1ToL2MessageTree.nextAvailableLeafIndex,
          "Invalid l1ToL2MessageTree.nextAvailableLeafIndex"
        );
        assertEq(
          header.stateReference.l1ToL2MessageTree.root,
          stateReference.l1ToL2MessageTree.root,
          "Invalid l1ToL2MessageTree.root"
        );

        // PartialStateReference
        {
          DecoderBase.PartialStateReference memory partialStateReference =
            referenceHeader.stateReference.partialStateReference;

          // NoteHashTree
          assertEq(
            header.stateReference.partialStateReference.noteHashTree.nextAvailableLeafIndex,
            partialStateReference.noteHashTree.nextAvailableLeafIndex,
            "Invalid noteHashTree.nextAvailableLeafIndex"
          );
          assertEq(
            header.stateReference.partialStateReference.noteHashTree.root,
            partialStateReference.noteHashTree.root,
            "Invalid noteHashTree.root"
          );

          // NullifierTree
          assertEq(
            header.stateReference.partialStateReference.nullifierTree.nextAvailableLeafIndex,
            partialStateReference.nullifierTree.nextAvailableLeafIndex,
            "Invalid nullifierTree.nextAvailableLeafIndex"
          );
          assertEq(
            header.stateReference.partialStateReference.nullifierTree.root,
            partialStateReference.nullifierTree.root,
            "Invalid nullifierTree.root"
          );

          // PublicDataTree
          assertEq(
            header.stateReference.partialStateReference.publicDataTree.nextAvailableLeafIndex,
            partialStateReference.publicDataTree.nextAvailableLeafIndex,
            "Invalid publicDataTree.nextAvailableLeafIndex"
          );
          assertEq(
            header.stateReference.partialStateReference.publicDataTree.root,
            partialStateReference.publicDataTree.root,
            "Invalid publicDataTree.root"
          );
        }
      }

      assertEq(
        header.lastArchive.nextAvailableLeafIndex,
        referenceHeader.lastArchive.nextAvailableLeafIndex,
        "Invalid lastArchive.nextAvailableLeafIndex"
      );
      assertEq(
        header.lastArchive.root, referenceHeader.lastArchive.root, "Invalid lastArchive.root"
      );
    }

    // Txs
    {
      bytes32 txsEffectsHash = txsHelper.decode(data.block.body);
      assertEq(txsEffectsHash, data.block.txsEffectsHash, "Invalid txs effects hash");
      assertEq(
        txsEffectsHash,
        data.block.decodedHeader.contentCommitment.txsEffectsHash,
        "Invalid txs effects hash"
      );
    }

    // The public inputs are computed based of these values, but not directly part of the decoding per say.
  }

  function testComputeKernelLogsIterationWithoutLogs() public {
    bytes memory kernelLogsLength = hex"00000004"; // 4 bytes containing value 4
    bytes memory iterationLogsLength = hex"00000000"; // 4 empty bytes indicating that length of this iteration's logs is 0
    bytes memory encodedLogs = abi.encodePacked(kernelLogsLength, iterationLogsLength);

    (bytes32 logsHash, uint256 bytesAdvanced) = txsHelper.computeKernelLogsHash(encodedLogs);

    bytes32 kernelPublicInputsLogsHash = bytes32(0);
    bytes32 privateCircuitPublicInputsLogsHash = Hash.sha256ToField(new bytes(0));

    bytes32 referenceLogsHash = Hash.sha256ToField(
      abi.encodePacked(kernelPublicInputsLogsHash, privateCircuitPublicInputsLogsHash)
    );

    assertEq(bytesAdvanced, encodedLogs.length, "Advanced by an incorrect number of bytes");
    assertEq(logsHash, referenceLogsHash, "Incorrect logs hash");
  }

  function testComputeKernelLogs1Iteration() public {
    // || K_LOGS_LEN | I1_LOGS_LEN | I1_LOGS ||
    // K_LOGS_LEN = 4 + 8 = 12 (hex"0000000c")
    // I1_LOGS_LEN = 8 (hex"00000008")
    // I1_LOGS = 8 bytes (hex"0000000493e78a70") // Note: 00000004 is the length of 1 log within function logs
    bytes memory firstFunctionCallLogs = hex"0000000493e78a70";
    // Prefix logs with length of kernel logs (12) and length of iteration 1 logs (8)
    bytes memory encodedLogs = abi.encodePacked(hex"0000000c00000008", firstFunctionCallLogs);
    (bytes32 logsHash, uint256 bytesAdvanced) = txsHelper.computeKernelLogsHash(encodedLogs);

    // Zero because this is the first iteration
    bytes32 previousKernelPublicInputsLogsHash = bytes32(0);
    bytes32 privateCircuitPublicInputsLogsHashFirstCall = Hash.sha256ToField(firstFunctionCallLogs);

    bytes32 referenceLogsHash = Hash.sha256ToField(
      abi.encodePacked(
        previousKernelPublicInputsLogsHash, privateCircuitPublicInputsLogsHashFirstCall
      )
    );

    assertEq(bytesAdvanced, encodedLogs.length, "Advanced by an incorrect number of bytes");
    assertEq(logsHash, referenceLogsHash, "Incorrect logs hash");
  }

  function testComputeKernelLogs2Iterations() public {
    // || K_LOGS_LEN | I1_LOGS_LEN | I1_LOGS | I2_LOGS_LEN | I2_LOGS ||
    // K_LOGS_LEN = 4 + 8 + 4 + 20 = 36 (hex"00000024")
    // I1_LOGS_LEN = 8 (hex"00000008")
    // I1_LOGS = 8 random bytes (hex"0000000493e78a70")
    // I2_LOGS_LEN = 20 (hex"00000014")
    // I2_LOGS = 20 bytes (hex"0000001006a86173c86c6d3f108eefc36e7fb014")
    bytes memory firstFunctionCallLogs = hex"0000000493e78a70";
    bytes memory secondFunctionCallLogs = hex"0000001006a86173c86c6d3f108eefc36e7fb014";
    bytes memory encodedLogs = abi.encodePacked(
      hex"0000002400000008", firstFunctionCallLogs, hex"00000014", secondFunctionCallLogs
    );
    (bytes32 logsHash, uint256 bytesAdvanced) = txsHelper.computeKernelLogsHash(encodedLogs);

    bytes32 referenceLogsHashFromIteration1 =
      Hash.sha256ToField(abi.encodePacked(bytes32(0), Hash.sha256ToField(firstFunctionCallLogs)));

    bytes32 privateCircuitPublicInputsLogsHashSecondCall =
      Hash.sha256ToField(secondFunctionCallLogs);

    bytes32 referenceLogsHashFromIteration2 = Hash.sha256ToField(
      abi.encodePacked(
        referenceLogsHashFromIteration1, privateCircuitPublicInputsLogsHashSecondCall
      )
    );

    assertEq(bytesAdvanced, encodedLogs.length, "Advanced by an incorrect number of bytes");
    assertEq(logsHash, referenceLogsHashFromIteration2, "Incorrect logs hash");
  }

  function testComputeKernelLogsMiddleIterationWithoutLogs() public {
    // || K_LOGS_LEN | I1_LOGS_LEN | I1_LOGS | I2_LOGS_LEN | I2_LOGS | I3_LOGS_LEN | I3_LOGS ||
    // K_LOGS_LEN = 4 + 8 + 4 + 0 + 4 + 20 = 40 (hex"00000028")
    // I1_LOGS_LEN = 8 (hex"00000008")
    // I1_LOGS = 8 random bytes (hex"0000000493e78a70")
    // I2_LOGS_LEN = 0 (hex"00000000")
    // I2_LOGS = 0 bytes (hex"")
    // I3_LOGS_LEN = 20 (hex"00000014")
    // I3_LOGS = 20 random bytes (hex"0000001006a86173c86c6d3f108eefc36e7fb014")
    bytes memory firstFunctionCallLogs = hex"0000000493e78a70";
    bytes memory secondFunctionCallLogs = hex"";
    bytes memory thirdFunctionCallLogs = hex"0000001006a86173c86c6d3f108eefc36e7fb014";
    bytes memory encodedLogs = abi.encodePacked(
      hex"0000002800000008",
      firstFunctionCallLogs,
      hex"00000000",
      secondFunctionCallLogs,
      hex"00000014",
      thirdFunctionCallLogs
    );
    (bytes32 logsHash, uint256 bytesAdvanced) = txsHelper.computeKernelLogsHash(encodedLogs);

    bytes32 referenceLogsHashFromIteration1 =
      Hash.sha256ToField(abi.encodePacked(bytes32(0), Hash.sha256ToField(firstFunctionCallLogs)));

    bytes32 privateCircuitPublicInputsLogsHashSecondCall =
      Hash.sha256ToField(secondFunctionCallLogs);

    bytes32 referenceLogsHashFromIteration2 = Hash.sha256ToField(
      abi.encodePacked(
        referenceLogsHashFromIteration1, privateCircuitPublicInputsLogsHashSecondCall
      )
    );

    bytes32 privateCircuitPublicInputsLogsHashThirdCall = Hash.sha256ToField(thirdFunctionCallLogs);

    bytes32 referenceLogsHashFromIteration3 = Hash.sha256ToField(
      abi.encodePacked(referenceLogsHashFromIteration2, privateCircuitPublicInputsLogsHashThirdCall)
    );

    assertEq(bytesAdvanced, encodedLogs.length, "Advanced by an incorrect number of bytes");
    assertEq(logsHash, referenceLogsHashFromIteration3, "Incorrect logs hash");
  }

  function testTxsDecoderCorrectlyComputesNumTxEffectsToPad() public {
    // Minimum num txs is 2 so when there are no real txs we need to pad to 2
    uint32 numTxEffects = 0;
    uint32 paddedNumTxEffects = txsHelper.computeNumTxEffectsToPad(numTxEffects);
    assertEq(paddedNumTxEffects, 2, "Incorrect number of tx effects to pad");

    numTxEffects = 1;
    paddedNumTxEffects = txsHelper.computeNumTxEffectsToPad(numTxEffects);
    assertEq(paddedNumTxEffects, 2 ** 1 - numTxEffects, "Incorrect number of tx effects to pad");

    numTxEffects = 3;
    paddedNumTxEffects = txsHelper.computeNumTxEffectsToPad(numTxEffects);
    assertEq(paddedNumTxEffects, 2 ** 2 - numTxEffects, "Incorrect number of tx effects to pad");

    numTxEffects = 5;
    paddedNumTxEffects = txsHelper.computeNumTxEffectsToPad(numTxEffects);
    assertEq(paddedNumTxEffects, 2 ** 3 - numTxEffects, "Incorrect number of tx effects to pad");

    numTxEffects = 8;
    paddedNumTxEffects = txsHelper.computeNumTxEffectsToPad(numTxEffects);
    assertEq(paddedNumTxEffects, 2 ** 3 - numTxEffects, "Incorrect number of tx effects to pad");

    numTxEffects = 10;
    paddedNumTxEffects = txsHelper.computeNumTxEffectsToPad(numTxEffects);
    assertEq(paddedNumTxEffects, 2 ** 4 - numTxEffects, "Incorrect number of tx effects to pad");

    numTxEffects = 16;
    paddedNumTxEffects = txsHelper.computeNumTxEffectsToPad(numTxEffects);
    assertEq(paddedNumTxEffects, 2 ** 4 - numTxEffects, "Incorrect number of tx effects to pad");

    numTxEffects = 17;
    paddedNumTxEffects = txsHelper.computeNumTxEffectsToPad(numTxEffects);
    assertEq(paddedNumTxEffects, 2 ** 5 - numTxEffects, "Incorrect number of tx effects to pad");
  }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DecoderBase} from "./Base.sol";

import {Hash} from "../../src/core/libraries/Hash.sol";
import {DataStructures} from "../../src/core/libraries/DataStructures.sol";
import {DecoderHelper} from "../DecoderHelper.sol";

import {Decoder} from "../../src/core/libraries/decoders/Decoder.sol";
import {HeaderDecoder} from "../../src/core/libraries/decoders/HeaderDecoder.sol";
import {MessagesDecoder} from "../../src/core/libraries/decoders/MessagesDecoder.sol";
import {TxsDecoder} from "../../src/core/libraries/decoders/TxsDecoder.sol";

import {AvailabilityOracle} from "../../src/core/availability_oracle/AvailabilityOracle.sol";

contract HeaderDecoderHelper {
  // A wrapper used such that we get "calldata" and not memory
  function decode(bytes calldata _header)
    public
    pure
    returns (uint256 l2BlockNumber, bytes32 startStateHash, bytes32 endStateHash)
  {
    return HeaderDecoder.decode(_header);
  }
}

contract MessagesDecoderHelper {
  // A wrapper used such that we get "calldata" and not memory
  function decode(bytes calldata _header)
    public
    pure
    returns (
      bytes32 l1ToL2MsgsHash,
      bytes32 l2ToL1MsgsHash,
      bytes32[] memory l1ToL2Msgs,
      bytes32[] memory l2ToL1Msgs
    )
  {
    return MessagesDecoder.decode(_header[HeaderDecoder.BLOCK_HEADER_SIZE:]);
  }
}

contract TxsDecoderHelper {
  // A wrapper used such that we get "calldata" and not memory
  function decode(bytes calldata _header) public pure returns (bytes32 txsHash) {
    return TxsDecoder.decode(_header[HeaderDecoder.BLOCK_HEADER_SIZE:]);
  }
}

/**
 * Blocks are generated using the `integration_l1_publisher.test.ts` tests.
 * Main use of these test is shorter cycles when updating the decoder contract.
 * All tests here are skipped (all tests are prefixed with an underscore)!
 * This is because we implicitly test the decoding in integration_l1_publisher.test.ts
 */
contract DecoderTest is DecoderBase {
  DecoderHelper internal helper;
  HeaderDecoderHelper internal headerHelper;
  MessagesDecoderHelper internal messagesHelper;
  TxsDecoderHelper internal txsHelper;

  function setUp() public virtual {
    helper = new DecoderHelper();
    headerHelper = new HeaderDecoderHelper();
    messagesHelper = new MessagesDecoderHelper();
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

    // Using the FULL decoder.
    (
      uint256 l2BlockNumber,
      bytes32 startStateHash,
      bytes32 endStateHash,
      bytes32 publicInputsHash,
      bytes32[] memory l2ToL1Msgs,
      bytes32[] memory l1ToL2Msgs
    ) = helper.decode(data.block.body);
    (bytes32 diffRoot, bytes32 l1ToL2MessagesHash) =
      helper.computeDiffRootAndMessagesHash(data.block.body);

    // Header
    {
      (uint256 headerL2BlockNumber, bytes32 headerStartStateHash, bytes32 headerEndStateHash) =
        headerHelper.decode(data.block.body);

      assertEq(l2BlockNumber, data.block.blockNumber, "Invalid block number");
      assertEq(headerL2BlockNumber, data.block.blockNumber, "Invalid block number");
      assertEq(startStateHash, data.block.startStateHash, "Invalid start state hash");
      assertEq(headerStartStateHash, data.block.startStateHash, "Invalid start state hash");
      assertEq(endStateHash, data.block.endStateHash, "Invalid end state hash");
      assertEq(headerEndStateHash, data.block.endStateHash, "Invalid end state hash");
    }

    // Messages
    {
      (
        bytes32 msgsInHash,
        bytes32 msgsL2ToL1MsgsHash,
        bytes32[] memory msgsL1ToL2Msgs,
        bytes32[] memory msgsL2ToL1Msgs
      ) = messagesHelper.decode(data.block.body);

      assertEq(msgsInHash, data.block.l1ToL2MessagesHash, "Invalid l1ToL2MsgsHash msgs");
      assertEq(l1ToL2MessagesHash, data.block.l1ToL2MessagesHash, "Invalid l1ToL2MsgsHash full");

      // assertEq(msgsL2ToL1MsgsHash, b.l2ToL1MessagesHash, "Invalid l2ToL1MsgsHash");

      // L1 -> L2 messages
      assertEq(
        msgsL1ToL2Msgs.length, data.messages.l1ToL2Messages.length, "Invalid l1ToL2Msgs length"
      );
      assertEq(l1ToL2Msgs.length, data.messages.l1ToL2Messages.length, "Invalid l1ToL2Msgs length");
      for (uint256 i = 0; i < msgsL1ToL2Msgs.length; i++) {
        assertEq(msgsL1ToL2Msgs[i], data.messages.l1ToL2Messages[i], "Invalid l1ToL2Msgs messages");
        assertEq(l1ToL2Msgs[i], data.messages.l1ToL2Messages[i], "Invalid l1ToL2Msgs full");
      }

      // L2 -> L1 messages
      assertEq(
        msgsL2ToL1Msgs.length, data.messages.l2ToL1Messages.length, "Invalid l2ToL1Msgs length"
      );
      assertEq(l2ToL1Msgs.length, data.messages.l2ToL1Messages.length, "Invalid l2ToL1Msgs length");
      for (uint256 i = 0; i < msgsL2ToL1Msgs.length; i++) {
        assertEq(msgsL2ToL1Msgs[i], data.messages.l2ToL1Messages[i], "Invalid l2ToL1Msgs messages");
        assertEq(l2ToL1Msgs[i], data.messages.l2ToL1Messages[i], "Invalid l2ToL1Msgs full");
      }
    }

    // Txs
    {
      bytes32 txsHash = txsHelper.decode(data.block.body);
      assertEq(txsHash, data.block.calldataHash, "Invalid txs hash");
      assertEq(diffRoot, data.block.calldataHash, "Invalid diff root/calldata hash");
    }

    // The public inputs are computed based of these values, but not directly part of the decoding per say.
  }

  function testComputeKernelLogsIterationWithoutLogs() public {
    bytes memory kernelLogsLength = hex"00000004"; // 4 bytes containing value 4
    bytes memory iterationLogsLength = hex"00000000"; // 4 empty bytes indicating that length of this iteration's logs is 0
    bytes memory encodedLogs = abi.encodePacked(kernelLogsLength, iterationLogsLength);

    (bytes32 logsHash, uint256 bytesAdvanced) = helper.computeKernelLogsHash(encodedLogs);

    bytes32 kernelPublicInputsLogsHash = bytes32(0);
    bytes32 privateCircuitPublicInputsLogsHash = sha256(new bytes(0));

    bytes32 referenceLogsHash =
      sha256(abi.encodePacked(kernelPublicInputsLogsHash, privateCircuitPublicInputsLogsHash));

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
    (bytes32 logsHash, uint256 bytesAdvanced) = helper.computeKernelLogsHash(encodedLogs);

    // Zero because this is the first iteration
    bytes32 previousKernelPublicInputsLogsHash = bytes32(0);
    bytes32 privateCircuitPublicInputsLogsHashFirstCall = sha256(firstFunctionCallLogs);

    bytes32 referenceLogsHash = sha256(
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
    (bytes32 logsHash, uint256 bytesAdvanced) = helper.computeKernelLogsHash(encodedLogs);

    bytes32 referenceLogsHashFromIteration1 =
      sha256(abi.encodePacked(bytes32(0), sha256(firstFunctionCallLogs)));

    bytes32 privateCircuitPublicInputsLogsHashSecondCall = sha256(secondFunctionCallLogs);

    bytes32 referenceLogsHashFromIteration2 = sha256(
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
    (bytes32 logsHash, uint256 bytesAdvanced) = helper.computeKernelLogsHash(encodedLogs);

    bytes32 referenceLogsHashFromIteration1 =
      sha256(abi.encodePacked(bytes32(0), sha256(firstFunctionCallLogs)));

    bytes32 privateCircuitPublicInputsLogsHashSecondCall = sha256(secondFunctionCallLogs);

    bytes32 referenceLogsHashFromIteration2 = sha256(
      abi.encodePacked(
        referenceLogsHashFromIteration1, privateCircuitPublicInputsLogsHashSecondCall
      )
    );

    bytes32 privateCircuitPublicInputsLogsHashThirdCall = sha256(thirdFunctionCallLogs);

    bytes32 referenceLogsHashFromIteration3 = sha256(
      abi.encodePacked(referenceLogsHashFromIteration2, privateCircuitPublicInputsLogsHashThirdCall)
    );

    assertEq(bytesAdvanced, encodedLogs.length, "Advanced by an incorrect number of bytes");
    assertEq(logsHash, referenceLogsHashFromIteration3, "Incorrect logs hash");
  }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

import {DecoderTest} from "./decoders/Decoder.t.sol";
import {DecoderHelper} from "./DecoderHelper.sol";

import {DecoderBase} from "./decoders/Base.sol";

import {DataStructures} from "../src/core/libraries/DataStructures.sol";

import {Registry} from "../src/core/messagebridge/Registry.sol";
import {Inbox} from "../src/core/messagebridge/Inbox.sol";
import {Outbox} from "../src/core/messagebridge/Outbox.sol";
import {Errors} from "../src/core/libraries/Errors.sol";
import {Rollup} from "../src/core/Rollup.sol";

/**
 * Blocks are generated using the `integration_l1_publisher.test.ts` tests.
 * Main use of these test is shorter cycles when updating the decoder contract.
 */
contract RollupTest is DecoderBase {
  DecoderHelper internal helper;
  Registry internal registry;
  Inbox internal inbox;
  Outbox internal outbox;
  Rollup internal rollup;

  function setUp() public virtual {
    helper = new DecoderHelper();

    registry = new Registry();
    inbox = new Inbox(address(registry));
    outbox = new Outbox(address(registry));
    rollup = new Rollup(registry);

    registry.upgrade(address(rollup), address(inbox), address(outbox));
  }

  function testMixedBlock() public {
    _testBlock("mixed_block_0");
  }

  function testConsecutiveMixedBlocks() public {
    _testBlock("mixed_block_0");
    _testBlock("mixed_block_1");
  }

  function testEmptyBlock() public {
    _testBlock("empty_block_0");
  }

  function testConsecutiveEmptyBlocks() public {
    _testBlock("empty_block_0");
    _testBlock("empty_block_1");
  }

  function testRevertInvalidChainId() public {
    bytes memory block_ = load("empty_block_0").block.body;

    assembly {
      mstore(add(block_, 0x20), 0x420)
    }

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__InvalidChainId.selector, 0x420, 31337));
    rollup.process(bytes(""), block_);
  }

  function testRevertInvalidVersion() public {
    bytes memory block_ = load("empty_block_0").block.body;

    assembly {
      mstore(add(block_, 0x40), 0x420)
    }

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__InvalidVersion.selector, 0x420, 1));
    rollup.process(bytes(""), block_);
  }

  function testRevertTimestampInFuture() public {
    bytes memory block_ = load("empty_block_0").block.body;

    uint256 ts = block.timestamp + 1;
    assembly {
      mstore(add(block_, 0x80), ts)
    }

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__TimestampInFuture.selector));
    rollup.process(bytes(""), block_);
  }

  function testRevertTimestampTooOld() public {
    bytes memory block_ = load("empty_block_0").block.body;

    // Overwrite in the rollup contract
    vm.store(address(rollup), bytes32(uint256(1)), bytes32(uint256(block.timestamp)));

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__TimestampTooOld.selector));
    rollup.process(bytes(""), block_);
  }

  function _testBlock(string memory name) public {
    DecoderBase.Full memory full = load(name);
    // We jump to the time of the block.
    vm.warp(full.block.timestamp);

    _populateInbox(full.populate.sender, full.populate.recipient, full.populate.l1ToL2Content);

    for (uint256 i = 0; i < full.messages.l1ToL2Messages.length; i++) {
      if (full.messages.l1ToL2Messages[i] == bytes32(0)) {
        continue;
      }
      assertTrue(inbox.contains(full.messages.l1ToL2Messages[i]), "msg not in inbox");
    }

    vm.record();
    rollup.process(bytes(""), full.block.body);

    (, bytes32[] memory inboxWrites) = vm.accesses(address(inbox));
    (, bytes32[] memory outboxWrites) = vm.accesses(address(outbox));

    {
      uint256 count = 0;
      for (uint256 i = 0; i < full.messages.l2ToL1Messages.length; i++) {
        if (full.messages.l2ToL1Messages[i] == bytes32(0)) {
          continue;
        }
        assertTrue(outbox.contains(full.messages.l2ToL1Messages[i]), "msg not in outbox");
        count++;
      }
      assertEq(outboxWrites.length, count, "Invalid outbox writes");
    }

    {
      uint256 count = 0;
      for (uint256 i = 0; i < full.messages.l1ToL2Messages.length; i++) {
        if (full.messages.l1ToL2Messages[i] == bytes32(0)) {
          continue;
        }
        assertFalse(inbox.contains(full.messages.l1ToL2Messages[i]), "msg not consumed");
        count++;
      }
      assertEq(inboxWrites.length, count, "Invalid inbox writes");
    }

    assertEq(rollup.rollupStateHash(), full.block.endStateHash, "Invalid rollup state hash");
  }

  function _populateInbox(address _sender, bytes32 _recipient, bytes32[] memory _contents) internal {
    uint32 deadline = type(uint32).max;
    for (uint256 i = 0; i < _contents.length; i++) {
      vm.prank(_sender);
      inbox.sendL2Message(
        DataStructures.L2Actor({actor: _recipient, version: 1}), deadline, _contents[i], bytes32(0)
      );
    }
  }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";
import {IOutbox} from "@aztec/core/interfaces/messagebridge/IOutbox.sol";
import {Outbox} from "@aztec/core/messagebridge/Outbox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";
import {Errors} from "@aztec/core/libraries/Errors.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {MessageBox} from "@aztec/core/libraries/MessageBox.sol";

contract OutboxTest is Test {
  Registry internal registry;
  Outbox internal outbox;
  uint256 internal version = 0;

  event MessageAdded(bytes32 indexed entryKey);
  event MessageConsumed(bytes32 indexed entryKey, address indexed recipient);

  function setUp() public {
    address rollup = address(this);
    registry = new Registry();
    outbox = new Outbox(address(registry));
    version = registry.upgrade(rollup, address(0x0), address(outbox));
  }

  function _fakeMessage() internal view returns (DataStructures.L2ToL1Msg memory) {
    return DataStructures.L2ToL1Msg({
      sender: DataStructures.L2Actor({
        actor: 0x2000000000000000000000000000000000000000000000000000000000000000,
        version: version
      }),
      recipient: DataStructures.L1Actor({actor: address(this), chainId: block.chainid}),
      content: 0x3000000000000000000000000000000000000000000000000000000000000000
    });
  }

  function testRevertIfInsertingFromNonRollup() public {
    vm.prank(address(0x1));
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = bytes32("random");
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Registry__RollupNotRegistered.selector, address(1))
    );
    outbox.sendL1Messages(entryKeys);
  }

  // fuzz batch insert -> check inserted. event emitted
  function testFuzzBatchInsert(bytes32[] memory _entryKeys) public {
    // expected events
    for (uint256 i = 0; i < _entryKeys.length; i++) {
      if (_entryKeys[i] == bytes32(0)) continue;
      vm.expectEmit(true, false, false, false);
      emit MessageAdded(_entryKeys[i]);
    }

    outbox.sendL1Messages(_entryKeys);
    for (uint256 i = 0; i < _entryKeys.length; i++) {
      if (_entryKeys[i] == bytes32(0)) continue;
      bytes32 key = _entryKeys[i];
      DataStructures.Entry memory entry = outbox.get(key);
      assertGt(entry.count, 0);
      assertEq(entry.fee, 0);
      assertEq(entry.deadline, 0);
    }
  }

  function testRevertIfConsumingFromWrongRecipient() public {
    DataStructures.L2ToL1Msg memory message = _fakeMessage();
    message.recipient.actor = address(0x1);
    vm.expectRevert(Errors.Outbox__Unauthorized.selector);
    outbox.consume(message);
  }

  function testRevertIfConsumingForWrongChain() public {
    DataStructures.L2ToL1Msg memory message = _fakeMessage();
    message.recipient.chainId = 2;
    vm.expectRevert(Errors.Outbox__InvalidChainId.selector);
    outbox.consume(message);
  }

  function testRevertIfConsumingMessageThatDoesntExist() public {
    DataStructures.L2ToL1Msg memory message = _fakeMessage();
    bytes32 entryKey = outbox.computeEntryKey(message);
    vm.expectRevert(abi.encodeWithSelector(Errors.Outbox__NothingToConsume.selector, entryKey));
    outbox.consume(message);
  }

  function testRevertIfInsertingFromWrongRollup() public {
    address wrongRollup = address(0xbeeffeed);
    uint256 wrongVersion = registry.upgrade(wrongRollup, address(0x0), address(outbox));

    DataStructures.L2ToL1Msg memory message = _fakeMessage();
    // correctly set message.recipient to this address
    message.recipient = DataStructures.L1Actor({actor: address(this), chainId: block.chainid});

    bytes32 expectedEntryKey = outbox.computeEntryKey(message);
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = expectedEntryKey;

    vm.prank(wrongRollup);
    outbox.sendL1Messages(entryKeys);

    vm.prank(message.recipient.actor);
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Outbox__InvalidVersion.selector, wrongVersion, version)
    );
    outbox.consume(message);
  }

  function testFuzzConsume(DataStructures.L2ToL1Msg memory _message) public {
    // correctly set message.recipient to this address
    _message.recipient = DataStructures.L1Actor({actor: address(this), chainId: block.chainid});

    // correctly set the message.sender.version
    _message.sender.version = version;

    bytes32 expectedEntryKey = outbox.computeEntryKey(_message);
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = expectedEntryKey;
    outbox.sendL1Messages(entryKeys);

    vm.prank(_message.recipient.actor);
    vm.expectEmit(true, true, false, false);
    emit MessageConsumed(expectedEntryKey, _message.recipient.actor);
    outbox.consume(_message);

    // ensure no such message to consume:
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Outbox__NothingToConsume.selector, expectedEntryKey)
    );
    outbox.consume(_message);
  }
}

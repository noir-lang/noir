// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";
import {IOutbox} from "@aztec/core/interfaces/messagebridge/IOutbox.sol";
import {Outbox} from "@aztec/core/messagebridge/Outbox.sol";
import {IMessageBox} from "@aztec/core/interfaces/messagebridge/IMessageBox.sol";
import {MessageBox} from "@aztec/core/messagebridge/MessageBox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";

import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";

contract OutboxTest is Test {
  Outbox outbox;

  event MessageAdded(bytes32 indexed entryKey);
  event MessageConsumed(bytes32 indexed entryKey, address indexed recipient);

  function setUp() public {
    address rollup = address(this);
    Registry registry = new Registry();
    outbox = new Outbox(address(registry));
    registry.setAddresses(rollup, address(0x0), address(outbox));
  }

  function _helper_computeEntryKey(DataStructures.L2ToL1Msg memory message)
    internal
    pure
    returns (bytes32)
  {
    return bytes32(
      uint256(sha256(abi.encode(message.sender, message.recipient, message.content)))
        % 21888242871839275222246405745257275088548364400416034343698204186575808495617
    );
  }

  function _fakeMessage() internal view returns (DataStructures.L2ToL1Msg memory) {
    return DataStructures.L2ToL1Msg({
      sender: DataStructures.L2Actor({
        actor: 0x2000000000000000000000000000000000000000000000000000000000000000,
        version: 1
      }),
      recipient: DataStructures.L1Actor({actor: address(this), chainId: block.chainid}),
      content: 0x3000000000000000000000000000000000000000000000000000000000000000
    });
  }

  function testRevertIfInsertingFromNonRollup() public {
    vm.prank(address(0x1));
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = bytes32("random");
    vm.expectRevert(MessageBox.MessageBox__Unauthorized.selector);
    outbox.sendL1Messages(entryKeys);
  }

  // fuzz batch insert -> check inserted. event emitted
  function testFuzzBatchInsert(bytes32[] memory entryKeys) public {
    // expected events
    for (uint256 i = 0; i < entryKeys.length; i++) {
      vm.expectEmit(true, false, false, false);
      emit MessageAdded(entryKeys[i]);
    }

    outbox.sendL1Messages(entryKeys);
    for (uint256 i = 0; i < entryKeys.length; i++) {
      bytes32 key = entryKeys[i];
      DataStructures.Entry memory entry = outbox.get(key);
      assertGt(entry.count, 0);
      assertEq(entry.fee, 0);
      assertEq(entry.deadline, 0);
    }
  }

  function testRevertIfConsumingFromWrongRecipient() public {
    DataStructures.L2ToL1Msg memory message = _fakeMessage();
    message.recipient.actor = address(0x1);
    vm.expectRevert(Outbox.Outbox__Unauthorized.selector);
    outbox.consume(message);
  }

  function testRevertIfConsumingForWrongChain() public {
    DataStructures.L2ToL1Msg memory message = _fakeMessage();
    message.recipient.chainId = 2;
    vm.expectRevert(Outbox.Outbox__WrongChainId.selector);
    outbox.consume(message);
  }

  function testRevertIfConsumingMessageThatDoesntExist() public {
    DataStructures.L2ToL1Msg memory message = _fakeMessage();
    bytes32 entryKey = _helper_computeEntryKey(message);
    vm.expectRevert(
      abi.encodeWithSelector(MessageBox.MessageBox__NothingToConsume.selector, entryKey)
    );
    outbox.consume(message);
  }

  function testConsume() public {
    // with fuzzing it takes way too long :()
    DataStructures.L2ToL1Msg memory message = _fakeMessage();
    bytes32 expectedEntryKey = _helper_computeEntryKey(message);
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = expectedEntryKey;
    outbox.sendL1Messages(entryKeys);

    vm.prank(message.recipient.actor);
    vm.expectEmit(true, true, false, false);
    emit MessageConsumed(expectedEntryKey, message.recipient.actor);
    outbox.consume(message);

    // ensure no such message to consume:
    vm.expectRevert(
      abi.encodeWithSelector(MessageBox.MessageBox__NothingToConsume.selector, expectedEntryKey)
    );
    outbox.get(expectedEntryKey);
  }
}

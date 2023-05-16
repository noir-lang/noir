// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {Inbox} from "@aztec/core/messagebridge/Inbox.sol";
import {IMessageBox} from "@aztec/core/interfaces/messagebridge/IMessageBox.sol";
import {MessageBox} from "@aztec/core/messagebridge/MessageBox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";

import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";

contract InboxTest is Test {
  Inbox inbox;

  event MessageAdded(
    bytes32 indexed entryKey,
    address indexed sender,
    bytes32 indexed recipient,
    uint256 senderChainId,
    uint256 recipientVersion,
    uint32 deadline,
    uint64 fee,
    bytes32 content
  );
  event L1ToL2MessageCancelled(bytes32 indexed entryKey);

  function setUp() public {
    address rollup = address(this);
    Registry registry = new Registry();
    inbox = new Inbox(address(registry));
    registry.setAddresses(rollup, address(inbox), address(0x0));
  }

  function _helper_computeEntryKey(DataStructures.L1ToL2Msg memory message)
    internal
    pure
    returns (bytes32)
  {
    return bytes32(
      uint256(
        sha256(
          abi.encode(
            message.sender,
            message.recipient,
            message.content,
            message.secretHash,
            message.deadline,
            message.fee
          )
        )
      ) % 21888242871839275222246405745257275088548364400416034343698204186575808495617
    );
  }

  function _fakeMessage() internal view returns (DataStructures.L1ToL2Msg memory) {
    return DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor({actor: address(this), chainId: block.chainid}),
      recipient: DataStructures.L2Actor({
        actor: 0x2000000000000000000000000000000000000000000000000000000000000000,
        version: 1
      }),
      content: 0x3000000000000000000000000000000000000000000000000000000000000000,
      secretHash: 0x4000000000000000000000000000000000000000000000000000000000000000,
      fee: 5,
      deadline: uint32(block.timestamp + 100)
    });
  }

  function testFuzzSendL2Msg(DataStructures.L1ToL2Msg memory message) public {
    // fix message.sender and deadline:
    message.sender = DataStructures.L1Actor({actor: address(this), chainId: block.chainid});
    if (message.deadline <= block.timestamp) {
      message.deadline = uint32(block.timestamp + 100);
    }
    bytes32 expectedEntryKey = _helper_computeEntryKey(message);
    vm.expectEmit(true, true, true, true);
    // event we expect
    emit MessageAdded(
      expectedEntryKey,
      message.sender.actor,
      message.recipient.actor,
      message.sender.chainId,
      message.recipient.version,
      message.deadline,
      message.fee,
      message.content
    );
    // event we will get
    bytes32 entryKey = inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    assertEq(entryKey, expectedEntryKey);
    assertEq(inbox.get(entryKey).count, 1);
    assertEq(inbox.get(entryKey).fee, message.fee);
    assertEq(inbox.get(entryKey).deadline, message.deadline);
  }

  function testSendMultipleSameL2Messages() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    bytes32 entryKey1 = inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    bytes32 entryKey2 = inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    bytes32 entryKey3 = inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );

    assertEq(entryKey1, entryKey2);
    assertEq(entryKey2, entryKey3);
    assertEq(inbox.get(entryKey1).count, 3);
    assertEq(inbox.get(entryKey1).fee, 5);
    assertEq(inbox.get(entryKey1).deadline, message.deadline);
  }

  function testRevertIfCancellingMessageFromDifferentAddress() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    vm.prank(address(0x1));
    vm.expectRevert(Inbox.Inbox__Unauthorized.selector);
    inbox.cancelL2Message(message, address(0x1));
  }

  function testRevertIfCancellingMessageWhenDeadlineHasntPassed() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    skip(1000); // block.timestamp now +1000 ms.
    vm.expectRevert(Inbox.Inbox__NotPastDeadline.selector);
    inbox.cancelL2Message(message, address(0x1));
  }

  function testRevertIfCancellingNonExistentMessage() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    bytes32 entryKey = _helper_computeEntryKey(message);
    vm.expectRevert(
      abi.encodeWithSelector(MessageBox.MessageBox__NothingToConsume.selector, entryKey)
    );
    inbox.cancelL2Message(message, address(0x1));
  }

  function testCancelMessage() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    address feeCollector = address(0x1);
    bytes32 expectedEntryKey = inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );

    vm.expectEmit(true, false, false, false);
    // event we expect
    emit L1ToL2MessageCancelled(expectedEntryKey);
    // event we will get
    inbox.cancelL2Message(message, feeCollector);

    // no such message to consume:
    vm.expectRevert(
      abi.encodeWithSelector(MessageBox.MessageBox__NothingToConsume.selector, expectedEntryKey)
    );
    inbox.get(expectedEntryKey);

    // fees accrued as expected:
    assertEq(inbox.feesAccrued(feeCollector), message.fee);
  }

  function testRevertIfNotConsumingFromRollup() public {
    vm.prank(address(0x1));
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = bytes32("random");
    vm.expectRevert(MessageBox.MessageBox__Unauthorized.selector);
    inbox.batchConsume(entryKeys, address(0x1));
  }

  function testRevertIfOneKeyIsPastDeadlineWhenBatchConsuming() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    bytes32 entryKey1 = inbox.sendL2Message{value: message.fee}(
      message.recipient, uint32(block.timestamp + 100), message.content, message.secretHash
    );
    bytes32 entryKey2 = inbox.sendL2Message{value: message.fee}(
      message.recipient, uint32(block.timestamp + 200), message.content, message.secretHash
    );
    bytes32 entryKey3 = inbox.sendL2Message{value: message.fee}(
      message.recipient, uint32(block.timestamp + 300), message.content, message.secretHash
    );
    bytes32[] memory entryKeys = new bytes32[](3);
    entryKeys[0] = entryKey1;
    entryKeys[1] = entryKey2;
    entryKeys[2] = entryKey3;

    skip(150); // block.timestamp now +150 ms. entryKey2 and entryKey3 is past deadline
    vm.expectRevert(Inbox.Inbox__PastDeadline.selector);
    inbox.batchConsume(entryKeys, address(0x1));
  }

  function testRevertIfConsumingAMessageThatDoesntExist(bytes32[] memory entryKeys) public {
    if (entryKeys.length == 0) {
      entryKeys = new bytes32[](1);
      entryKeys[0] = bytes32("random");
    }
    vm.expectRevert(
      abi.encodeWithSelector(MessageBox.MessageBox__NothingToConsume.selector, entryKeys[0])
    );
    inbox.batchConsume(entryKeys, address(0x1));
  }

  function testBatchConsume(DataStructures.L1ToL2Msg[] memory messages) public {
    bytes32[] memory entryKeys = new bytes32[](messages.length);
    uint256 expectedTotalFee = 0;
    address feeCollector = address(0x1);
    uint256 maxDeadline = 0; // for skipping time (to avoid past deadline revert)

    // insert messages:
    for (uint256 i = 0; i < messages.length; i++) {
      DataStructures.L1ToL2Msg memory message = messages[i];
      // fix message.sender and deadline:
      message.sender = DataStructures.L1Actor({actor: address(this), chainId: block.chainid});
      if (message.deadline <= block.timestamp) {
        message.deadline = uint32(block.timestamp + 100);
      }
      if (message.deadline > maxDeadline) {
        maxDeadline = message.deadline;
      }
      expectedTotalFee += message.fee;
      entryKeys[i] = inbox.sendL2Message{value: message.fee}(
        message.recipient, message.deadline, message.content, message.secretHash
      );
    }

    skip(maxDeadline + 100);
    // batch consume:
    inbox.batchConsume(entryKeys, feeCollector);

    // fees accrued as expected:
    assertEq(inbox.feesAccrued(feeCollector), expectedTotalFee);
  }
}

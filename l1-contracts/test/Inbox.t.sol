// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {Inbox} from "@aztec/core/messagebridge/Inbox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";
import {Constants} from "@aztec/core/libraries/ConstantsGen.sol";
import {Errors} from "@aztec/core/libraries/Errors.sol";

import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {MessageBox} from "@aztec/core/libraries/MessageBox.sol";

contract InboxTest is Test {
  event MessageAdded(
    bytes32 indexed entryKey,
    address indexed sender,
    bytes32 indexed recipient,
    uint256 senderChainId,
    uint256 recipientVersion,
    uint32 deadline,
    uint64 fee,
    bytes32 content,
    bytes32 secretHash
  );

  event L1ToL2MessageCancelled(bytes32 indexed entryKey);

  Registry internal registry;
  Inbox internal inbox;
  uint256 internal version = 0;

  function setUp() public {
    address rollup = address(this);
    registry = new Registry();
    inbox = new Inbox(address(registry));
    version = registry.upgrade(rollup, address(inbox), address(0x0));
  }

  function _fakeMessage() internal view returns (DataStructures.L1ToL2Msg memory) {
    return DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor({actor: address(this), chainId: block.chainid}),
      recipient: DataStructures.L2Actor({
        actor: 0x1000000000000000000000000000000000000000000000000000000000000000,
        version: version
      }),
      content: 0x2000000000000000000000000000000000000000000000000000000000000000,
      secretHash: 0x3000000000000000000000000000000000000000000000000000000000000000,
      fee: 5,
      deadline: uint32(block.timestamp + 100)
    });
  }

  function testFuzzSendL2Msg(DataStructures.L1ToL2Msg memory _message) public {
    // fix message.sender and deadline:
    _message.sender = DataStructures.L1Actor({actor: address(this), chainId: block.chainid});
    // ensure actor fits in a field
    _message.recipient.actor = bytes32(uint256(_message.recipient.actor) % Constants.P);
    if (_message.deadline <= block.timestamp) {
      _message.deadline = uint32(block.timestamp + 100);
    }
    // ensure content fits in a field
    _message.content = bytes32(uint256(_message.content) % Constants.P);
    // ensure secret hash fits in a field
    _message.secretHash = bytes32(uint256(_message.secretHash) % Constants.P);
    bytes32 expectedEntryKey = inbox.computeEntryKey(_message);
    vm.expectEmit(true, true, true, true);
    // event we expect
    emit MessageAdded(
      expectedEntryKey,
      _message.sender.actor,
      _message.recipient.actor,
      _message.sender.chainId,
      _message.recipient.version,
      _message.deadline,
      _message.fee,
      _message.content,
      _message.secretHash
    );
    // event we will get
    bytes32 entryKey = inbox.sendL2Message{value: _message.fee}(
      _message.recipient, _message.deadline, _message.content, _message.secretHash
    );
    assertEq(entryKey, expectedEntryKey);
    DataStructures.Entry memory entry = inbox.get(entryKey);
    assertEq(entry.count, 1);
    assertEq(entry.fee, _message.fee);
    assertEq(entry.deadline, _message.deadline);
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

  function testRevertIfActorTooLarge() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    message.recipient.actor = bytes32(Constants.MAX_FIELD_VALUE + 1);
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Inbox__ActorTooLarge.selector, message.recipient.actor)
    );
    inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
  }

  function testRevertIfContentTooLarge() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    message.content = bytes32(Constants.MAX_FIELD_VALUE + 1);
    vm.expectRevert(abi.encodeWithSelector(Errors.Inbox__ContentTooLarge.selector, message.content));
    inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
  }

  function testRevertIfSecretHashTooLarge() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    message.secretHash = bytes32(Constants.MAX_FIELD_VALUE + 1);
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Inbox__SecretHashTooLarge.selector, message.secretHash)
    );
    inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
  }

  function testRevertIfCancellingMessageFromDifferentAddress() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    vm.prank(address(0x1));
    vm.expectRevert(Errors.Inbox__Unauthorized.selector);
    inbox.cancelL2Message(message, address(0x1));
  }

  function testRevertIfCancellingMessageWhenDeadlineHasntPassed() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    message.deadline = uint32(block.timestamp + 1000);
    inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    skip(500); // deadline = 1000. block.timestamp = 500. Not cancellable:
    vm.expectRevert(Errors.Inbox__NotPastDeadline.selector);
    inbox.cancelL2Message(message, address(0x1));
  }

  function testRevertIfCancellingNonExistentMessage() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    bytes32 entryKey = inbox.computeEntryKey(message);
    skip(500); // make message cancellable.
    vm.expectRevert(abi.encodeWithSelector(Errors.Inbox__NothingToConsume.selector, entryKey));
    inbox.cancelL2Message(message, address(0x1));
  }

  function testCancelMessage() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    address feeCollector = address(0x1);
    bytes32 expectedEntryKey = inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    skip(500); // make message cancellable.

    vm.expectEmit(true, false, false, false);
    // event we expect
    emit L1ToL2MessageCancelled(expectedEntryKey);
    // event we will get
    inbox.cancelL2Message(message, feeCollector);
    // fees accrued as expected:
    assertEq(inbox.feesAccrued(feeCollector), message.fee);

    // no such message to consume:
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = expectedEntryKey;
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Inbox__NothingToConsume.selector, expectedEntryKey)
    );
    inbox.batchConsume(entryKeys, feeCollector);
  }

  function testRevertIfNotConsumingFromRollup() public {
    vm.prank(address(0x1));
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = bytes32("random");
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Registry__RollupNotRegistered.selector, address(1))
    );
    inbox.batchConsume(entryKeys, address(0x1));
  }

  function testRevertIfOneKeyIsPastDeadlineWhenBatchConsuming() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    bytes32 entryKey1 = inbox.sendL2Message{value: message.fee}(
      message.recipient, uint32(block.timestamp + 200), message.content, message.secretHash
    );
    bytes32 entryKey2 = inbox.sendL2Message{value: message.fee}(
      message.recipient, uint32(block.timestamp + 100), message.content, message.secretHash
    );
    bytes32 entryKey3 = inbox.sendL2Message{value: message.fee}(
      message.recipient, uint32(block.timestamp + 300), message.content, message.secretHash
    );
    bytes32[] memory entryKeys = new bytes32[](3);
    entryKeys[0] = entryKey1;
    entryKeys[1] = entryKey2;
    entryKeys[2] = entryKey3;

    skip(150); // block.timestamp now +150 ms. entryKey2 is past deadline
    vm.expectRevert(Errors.Inbox__PastDeadline.selector);
    inbox.batchConsume(entryKeys, address(0x1));
  }

  function testFuzzRevertIfConsumingAMessageThatDoesntExist(bytes32 _entryKey) public {
    bytes32[] memory entryKeys = new bytes32[](1);
    if (_entryKey == bytes32(0)) {
      entryKeys[0] = bytes32("random");
    } else {
      entryKeys[0] = _entryKey;
    }
    vm.expectRevert(abi.encodeWithSelector(Errors.Inbox__NothingToConsume.selector, entryKeys[0]));
    inbox.batchConsume(entryKeys, address(0x1));
  }

  function testRevertIfConsumingTheSameMessageMoreThanTheCountOfEntries() public {
    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    address feeCollector = address(0x1);
    bytes32 entryKey = inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = entryKey;

    inbox.batchConsume(entryKeys, feeCollector);
    assertEq(inbox.feesAccrued(feeCollector), message.fee);

    // consuming this again should fail:
    vm.expectRevert(abi.encodeWithSelector(Errors.Inbox__NothingToConsume.selector, entryKeys[0]));
    inbox.batchConsume(entryKeys, feeCollector);
  }

  function testRevertIfConsumingFromWrongRollup() public {
    address wrongRollup = address(0xbeeffeed);
    uint256 wrongVersion = registry.upgrade(wrongRollup, address(inbox), address(0x0));

    DataStructures.L1ToL2Msg memory message = _fakeMessage();
    address feeCollector = address(0x1);
    bytes32 entryKey = inbox.sendL2Message{value: message.fee}(
      message.recipient, message.deadline, message.content, message.secretHash
    );
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = entryKey;

    vm.prank(wrongRollup);
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Inbox__InvalidVersion.selector, version, wrongVersion)
    );
    inbox.batchConsume(entryKeys, feeCollector);
  }

  function testFuzzBatchConsume(DataStructures.L1ToL2Msg[] memory _messages) public {
    bytes32[] memory entryKeys = new bytes32[](_messages.length);
    uint256 expectedTotalFee = 0;
    address feeCollector = address(0x1);

    // insert messages:
    for (uint256 i = 0; i < _messages.length; i++) {
      DataStructures.L1ToL2Msg memory message = _messages[i];
      // fix message.sender and deadline to be more than current time:
      message.sender = DataStructures.L1Actor({actor: address(this), chainId: block.chainid});
      // ensure actor fits in a field
      message.recipient.actor = bytes32(uint256(message.recipient.actor) % Constants.P);
      if (message.deadline <= block.timestamp) {
        message.deadline = uint32(block.timestamp + 100);
      }
      // ensure content fits in a field
      message.content = bytes32(uint256(message.content) % Constants.P);
      // ensure secret hash fits in a field
      message.secretHash = bytes32(uint256(message.secretHash) % Constants.P);
      // update version
      message.recipient.version = version;
      expectedTotalFee += message.fee;
      entryKeys[i] = inbox.sendL2Message{value: message.fee}(
        message.recipient, message.deadline, message.content, message.secretHash
      );
    }

    // batch consume:
    inbox.batchConsume(entryKeys, feeCollector);

    // fees accrued as expected:
    assertEq(inbox.feesAccrued(feeCollector), expectedTotalFee);
  }
}

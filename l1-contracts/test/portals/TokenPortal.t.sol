pragma solidity >=0.8.18;

import "forge-std/Test.sol";

// Rollup Proccessor
import {Rollup} from "@aztec/core/Rollup.sol";
import {Inbox} from "@aztec/core/messagebridge/Inbox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";
import {Outbox} from "@aztec/core/messagebridge/Outbox.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {Hash} from "@aztec/core/libraries/Hash.sol";
import {Errors} from "@aztec/core/libraries/Errors.sol";

// Interfaces
import {IRegistry} from "@aztec/core/interfaces/messagebridge/IRegistry.sol";
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";

// Portal tokens
import {TokenPortal} from "./TokenPortal.sol";
import {PortalERC20} from "./PortalERC20.sol";

contract TokenPortalTest is Test {
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
  event MessageConsumed(bytes32 indexed entryKey, address indexed recipient);

  Registry internal registry;
  Inbox internal inbox;
  Outbox internal outbox;
  Rollup internal rollup;
  bytes32 internal l2TokenAddress = bytes32(uint256(0x42));

  TokenPortal internal tokenPortal;
  PortalERC20 internal portalERC20;

  // input params
  uint32 internal deadline = uint32(block.timestamp + 1 days);
  bytes32 internal to = bytes32(0x2d749407d8c364537cdeb799c1574929cb22ff1ece2b96d2a1c6fa287a0e0171);
  uint256 internal amount = 100;
  uint256 internal mintAmount = 1 ether;
  // this hash is just a random 32 byte string
  bytes32 internal secretHashForL2MessageConsumption =
    0x147e4fec49805c924e28150fc4b36824679bc17ecb1d7d9f6a9effb7fde6b6a0;
  // this hash is just a random 32 byte string
  bytes32 internal secretHashForRedeemingMintedNotes =
    0x157e4fec49805c924e28150fc4b36824679bc17ecb1d7d9f6a9effb7fde6b6a0;
  uint64 internal bid = 1 ether;

  // params for withdraw:
  address internal recipient = address(0xdead);
  uint256 internal withdrawAmount = 654;

  function setUp() public {
    registry = new Registry();
    inbox = new Inbox(address(registry));
    outbox = new Outbox(address(registry));
    rollup = new Rollup(registry);

    registry.upgrade(address(rollup), address(inbox), address(outbox));

    portalERC20 = new PortalERC20();
    tokenPortal = new TokenPortal();

    tokenPortal.initialize(address(registry), address(portalERC20), l2TokenAddress);

    vm.deal(address(this), 100 ether);
  }

  function _createExpectedMintPrivateL1ToL2Message(address _canceller)
    internal
    view
    returns (DataStructures.L1ToL2Msg memory)
  {
    return DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor(address(tokenPortal), block.chainid),
      recipient: DataStructures.L2Actor(l2TokenAddress, 1),
      content: Hash.sha256ToField(
        abi.encodeWithSignature(
          "mint_private(uint256,bytes32,address)",
          amount,
          secretHashForRedeemingMintedNotes,
          _canceller
        )
        ),
      secretHash: secretHashForL2MessageConsumption,
      deadline: deadline,
      fee: bid
    });
  }

  function _createExpectedMintPublicL1ToL2Message(address _canceller)
    internal
    view
    returns (DataStructures.L1ToL2Msg memory)
  {
    return DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor(address(tokenPortal), block.chainid),
      recipient: DataStructures.L2Actor(l2TokenAddress, 1),
      content: Hash.sha256ToField(
        abi.encodeWithSignature("mint_public(uint256,bytes32,address)", amount, to, _canceller)
        ),
      secretHash: secretHashForL2MessageConsumption,
      deadline: deadline,
      fee: bid
    });
  }

  function testDepositPrivate() public returns (bytes32) {
    // mint token and approve to the portal
    portalERC20.mint(address(this), mintAmount);
    portalERC20.approve(address(tokenPortal), mintAmount);

    // Check for the expected message
    DataStructures.L1ToL2Msg memory expectedMessage =
      _createExpectedMintPrivateL1ToL2Message(address(this));
    bytes32 expectedEntryKey = inbox.computeEntryKey(expectedMessage);

    // Check the event was emitted
    vm.expectEmit(true, true, true, true);
    // event we expect
    emit MessageAdded(
      expectedEntryKey,
      expectedMessage.sender.actor,
      expectedMessage.recipient.actor,
      expectedMessage.sender.chainId,
      expectedMessage.recipient.version,
      expectedMessage.deadline,
      expectedMessage.fee,
      expectedMessage.content,
      expectedMessage.secretHash
    );

    // Perform op
    bytes32 entryKey = tokenPortal.depositToAztecPrivate{value: bid}(
      amount,
      secretHashForRedeemingMintedNotes,
      address(this),
      deadline,
      secretHashForL2MessageConsumption
    );

    assertEq(entryKey, expectedEntryKey, "returned entry key and calculated entryKey should match");

    // Check that the message is in the inbox
    DataStructures.Entry memory entry = inbox.get(entryKey);
    assertEq(entry.count, 1);

    return entryKey;
  }

  function testDepositPublic() public returns (bytes32) {
    // mint token and approve to the portal
    portalERC20.mint(address(this), mintAmount);
    portalERC20.approve(address(tokenPortal), mintAmount);

    // Check for the expected message
    DataStructures.L1ToL2Msg memory expectedMessage =
      _createExpectedMintPublicL1ToL2Message(address(this));
    bytes32 expectedEntryKey = inbox.computeEntryKey(expectedMessage);

    // Check the event was emitted
    vm.expectEmit(true, true, true, true);
    // event we expect
    emit MessageAdded(
      expectedEntryKey,
      expectedMessage.sender.actor,
      expectedMessage.recipient.actor,
      expectedMessage.sender.chainId,
      expectedMessage.recipient.version,
      expectedMessage.deadline,
      expectedMessage.fee,
      expectedMessage.content,
      expectedMessage.secretHash
    );

    // Perform op
    bytes32 entryKey = tokenPortal.depositToAztecPublic{value: bid}(
      amount, to, address(this), deadline, secretHashForL2MessageConsumption
    );

    assertEq(entryKey, expectedEntryKey, "returned entry key and calculated entryKey should match");

    // Check that the message is in the inbox
    DataStructures.Entry memory entry = inbox.get(entryKey);
    assertEq(entry.count, 1);

    return entryKey;
  }

  function testCancelPublic() public {
    bytes32 expectedEntryKey = testDepositPublic();
    // now cancel the message - move time forward (post deadline)
    vm.warp(deadline + 1 days);

    // ensure no one else can cancel the message:
    vm.startPrank(address(0xdead));
    bytes32 expectedWrongEntryKey =
      inbox.computeEntryKey(_createExpectedMintPublicL1ToL2Message(address(0xdead)));
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Inbox__NothingToConsume.selector, expectedWrongEntryKey)
    );
    tokenPortal.cancelL1ToAztecMessagePublic(
      amount, to, deadline, secretHashForL2MessageConsumption, bid
    );
    vm.stopPrank();

    // ensure cant cancel with cancelPrivate (since deposit was public)
    expectedWrongEntryKey =
      inbox.computeEntryKey(_createExpectedMintPrivateL1ToL2Message(address(this)));
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Inbox__NothingToConsume.selector, expectedWrongEntryKey)
    );
    tokenPortal.cancelL1ToAztecMessagePrivate(
      amount, secretHashForRedeemingMintedNotes, deadline, secretHashForL2MessageConsumption, bid
    );

    // actually cancel the message
    // check event was emitted
    vm.expectEmit(true, false, false, false);
    // expected event:
    emit L1ToL2MessageCancelled(expectedEntryKey);
    // perform op
    bytes32 entryKey = tokenPortal.cancelL1ToAztecMessagePublic(
      amount, to, deadline, secretHashForL2MessageConsumption, bid
    );

    assertEq(entryKey, expectedEntryKey, "returned entry key and calculated entryKey should match");
    assertFalse(inbox.contains(entryKey), "entry still in inbox");
    assertEq(
      portalERC20.balanceOf(address(this)),
      mintAmount,
      "assets should be transferred back to this contract"
    );
    assertEq(portalERC20.balanceOf(address(tokenPortal)), 0, "portal should have no assets");
  }

  function testCancelPrivate() public {
    bytes32 expectedEntryKey = testDepositPrivate();
    // now cancel the message - move time forward (post deadline)
    vm.warp(deadline + 1 days);

    // ensure no one else can cancel the message:
    vm.startPrank(address(0xdead));
    bytes32 expectedWrongEntryKey =
      inbox.computeEntryKey(_createExpectedMintPrivateL1ToL2Message(address(0xdead)));
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Inbox__NothingToConsume.selector, expectedWrongEntryKey)
    );
    tokenPortal.cancelL1ToAztecMessagePrivate(
      amount, secretHashForRedeemingMintedNotes, deadline, secretHashForL2MessageConsumption, bid
    );
    vm.stopPrank();

    // ensure cant cancel with cancelPublic (since deposit was private)
    expectedWrongEntryKey =
      inbox.computeEntryKey(_createExpectedMintPublicL1ToL2Message(address(this)));
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Inbox__NothingToConsume.selector, expectedWrongEntryKey)
    );
    tokenPortal.cancelL1ToAztecMessagePublic(
      amount, to, deadline, secretHashForL2MessageConsumption, bid
    );

    // actually cancel the message
    // check event was emitted
    vm.expectEmit(true, false, false, false);
    // expected event:
    emit L1ToL2MessageCancelled(expectedEntryKey);
    // perform op
    bytes32 entryKey = tokenPortal.cancelL1ToAztecMessagePrivate(
      amount, secretHashForRedeemingMintedNotes, deadline, secretHashForL2MessageConsumption, bid
    );

    assertEq(entryKey, expectedEntryKey, "returned entry key and calculated entryKey should match");
    assertFalse(inbox.contains(entryKey), "entry still in inbox");
    assertEq(
      portalERC20.balanceOf(address(this)),
      mintAmount,
      "assets should be transferred back to this contract"
    );
    assertEq(portalERC20.balanceOf(address(tokenPortal)), 0, "portal should have no assets");
  }

  function _createWithdrawMessageForOutbox(address _designatedCaller)
    internal
    view
    returns (bytes32)
  {
    bytes32 entryKey = outbox.computeEntryKey(
      DataStructures.L2ToL1Msg({
        sender: DataStructures.L2Actor({actor: l2TokenAddress, version: 1}),
        recipient: DataStructures.L1Actor({actor: address(tokenPortal), chainId: block.chainid}),
        content: Hash.sha256ToField(
          abi.encodeWithSignature(
            "withdraw(uint256,address,address)", withdrawAmount, recipient, _designatedCaller
          )
          )
      })
    );
    return entryKey;
  }

  function _addWithdrawMessageInOutbox(address _designatedCaller) internal returns (bytes32) {
    // send assets to the portal
    portalERC20.mint(address(tokenPortal), withdrawAmount);

    // Create the message
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = _createWithdrawMessageForOutbox(_designatedCaller);
    // Insert messages into the outbox (impersonating the rollup contract)
    vm.prank(address(rollup));
    outbox.sendL1Messages(entryKeys);
    return entryKeys[0];
  }

  function testAnyoneCanCallWithdrawIfNoDesignatedCaller(address _caller) public {
    vm.assume(_caller != address(0));
    bytes32 expectedEntryKey = _addWithdrawMessageInOutbox(address(0));
    assertEq(portalERC20.balanceOf(recipient), 0);

    vm.startPrank(_caller);
    vm.expectEmit(true, true, true, true);
    emit MessageConsumed(expectedEntryKey, address(tokenPortal));
    bytes32 actualEntryKey = tokenPortal.withdraw(withdrawAmount, recipient, false);
    assertEq(expectedEntryKey, actualEntryKey);
    // Should have received 654 RNA tokens
    assertEq(portalERC20.balanceOf(recipient), withdrawAmount);

    // Should not be able to withdraw again
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Outbox__NothingToConsume.selector, actualEntryKey)
    );
    tokenPortal.withdraw(withdrawAmount, recipient, false);
    vm.stopPrank();
  }

  function testWithdrawWithDesignatedCallerFailsForOtherCallers(address _caller) public {
    vm.assume(_caller != address(this));
    // add message with caller as this address
    _addWithdrawMessageInOutbox(address(this));

    vm.startPrank(_caller);
    bytes32 entryKeyPortalChecksAgainst = _createWithdrawMessageForOutbox(_caller);
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Outbox__NothingToConsume.selector, entryKeyPortalChecksAgainst)
    );
    tokenPortal.withdraw(withdrawAmount, recipient, true);

    entryKeyPortalChecksAgainst = _createWithdrawMessageForOutbox(address(0));
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Outbox__NothingToConsume.selector, entryKeyPortalChecksAgainst)
    );
    tokenPortal.withdraw(withdrawAmount, recipient, false);
    vm.stopPrank();
  }

  function testWithdrawWithDesignatedCallerSucceedsForDesignatedCaller() public {
    // add message with caller as this address
    bytes32 expectedEntryKey = _addWithdrawMessageInOutbox(address(this));

    vm.expectEmit(true, true, true, true);
    emit MessageConsumed(expectedEntryKey, address(tokenPortal));
    bytes32 actualEntryKey = tokenPortal.withdraw(withdrawAmount, recipient, true);
    assertEq(expectedEntryKey, actualEntryKey);
    // Should have received 654 RNA tokens
    assertEq(portalERC20.balanceOf(recipient), withdrawAmount);
  }
}

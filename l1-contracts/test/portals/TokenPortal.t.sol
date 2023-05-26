pragma solidity >=0.8.18;

import "forge-std/Test.sol";

// Rollup Proccessor
import {Rollup} from "@aztec/core/Rollup.sol";
import {Inbox} from "@aztec/core/messagebridge/Inbox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";
import {Outbox} from "@aztec/core/messagebridge/Outbox.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {Hash} from "@aztec/core/libraries/Hash.sol";

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

  Registry internal registry;
  Inbox internal inbox;
  Outbox internal outbox;
  Rollup internal rollup;
  bytes32 internal l2TokenAddress = bytes32(uint256(0x42));

  TokenPortal tokenPortal;
  PortalERC20 portalERC20;

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

  function testDeposit() public {
    // mint token and approve to the portal
    portalERC20.mint(address(this), 1 ether);
    portalERC20.approve(address(tokenPortal), 1 ether);

    // input params
    uint32 deadline = uint32(block.timestamp + 1 days);
    bytes32 to = bytes32(0x2d749407d8c364537cdeb799c1574929cb22ff1ece2b96d2a1c6fa287a0e0171);
    uint256 amount = 100;
    bytes32 secretHash = 0x147e4fec49805c924e28150fc4b36824679bc17ecb1d7d9f6a9effb7fde6b6a0;
    uint64 bid = 1 ether;

    // Check for the expected message
    DataStructures.L1ToL2Msg memory expectedMessage = DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor(address(tokenPortal), block.chainid),
      recipient: DataStructures.L2Actor(l2TokenAddress, 1),
      content: Hash.sha256ToField(abi.encodeWithSignature("mint(uint256,bytes32)", amount, to)),
      secretHash: secretHash,
      deadline: deadline,
      fee: bid
    });
    bytes32 expectedEntryKey = inbox.computeEntryKey(expectedMessage);

    // Check the even was emitted
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
    bytes32 entryKey = tokenPortal.depositToAztec{value: bid}(to, amount, deadline, secretHash);

    assertEq(entryKey, expectedEntryKey, "returned entry key and calculated entryKey should match");

    // Check that the message is in the inbox
    DataStructures.Entry memory entry = inbox.get(entryKey);
    assertEq(entry.count, 1);
  }
}

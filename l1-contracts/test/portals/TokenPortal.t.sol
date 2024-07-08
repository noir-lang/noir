pragma solidity >=0.8.18;

import "forge-std/Test.sol";

// Rollup Processor
import {Rollup} from "../../src/core/Rollup.sol";
import {AvailabilityOracle} from "../../src/core/availability_oracle/AvailabilityOracle.sol";
import {Constants} from "../../src/core/libraries/ConstantsGen.sol";
import {Registry} from "../../src/core/messagebridge/Registry.sol";
import {DataStructures} from "../../src/core/libraries/DataStructures.sol";
import {Hash} from "../../src/core/libraries/Hash.sol";
import {Errors} from "../../src/core/libraries/Errors.sol";

// Interfaces
import {IInbox} from "../../src/core/interfaces/messagebridge/IInbox.sol";
import {IOutbox} from "../../src/core/interfaces/messagebridge/IOutbox.sol";
import {IERC20} from "@oz/token/ERC20/IERC20.sol";

// Portal tokens
import {TokenPortal} from "./TokenPortal.sol";
import {PortalERC20} from "./PortalERC20.sol";

import {NaiveMerkle} from "../merkle/Naive.sol";

contract TokenPortalTest is Test {
  using Hash for DataStructures.L1ToL2Msg;

  uint256 internal constant FIRST_REAL_TREE_NUM = Constants.INITIAL_L2_BLOCK_NUM + 1;

  event MessageConsumed(bytes32 indexed messageHash, address indexed recipient);

  Registry internal registry;

  IInbox internal inbox;
  IOutbox internal outbox;

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

  // params for withdraw:
  address internal recipient = address(0xdead);
  uint256 internal withdrawAmount = 654;

  function setUp() public {
    registry = new Registry();
    portalERC20 = new PortalERC20();
    rollup =
      new Rollup(registry, new AvailabilityOracle(), IERC20(address(portalERC20)), bytes32(0));
    inbox = rollup.INBOX();
    outbox = rollup.OUTBOX();

    registry.upgrade(address(rollup), address(inbox), address(outbox));

    portalERC20.mint(address(rollup), 1000000);
    tokenPortal = new TokenPortal();

    tokenPortal.initialize(address(registry), address(portalERC20), l2TokenAddress);

    vm.deal(address(this), 100 ether);
  }

  function _createExpectedMintPrivateL1ToL2Message()
    internal
    view
    returns (DataStructures.L1ToL2Msg memory)
  {
    return DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor(address(tokenPortal), block.chainid),
      recipient: DataStructures.L2Actor(l2TokenAddress, 1),
      content: Hash.sha256ToField(
        abi.encodeWithSignature(
          "mint_private(bytes32,uint256)", secretHashForRedeemingMintedNotes, amount
        )
        ),
      secretHash: secretHashForL2MessageConsumption
    });
  }

  function _createExpectedMintPublicL1ToL2Message()
    internal
    view
    returns (DataStructures.L1ToL2Msg memory)
  {
    return DataStructures.L1ToL2Msg({
      sender: DataStructures.L1Actor(address(tokenPortal), block.chainid),
      recipient: DataStructures.L2Actor(l2TokenAddress, 1),
      content: Hash.sha256ToField(abi.encodeWithSignature("mint_public(bytes32,uint256)", to, amount)),
      secretHash: secretHashForL2MessageConsumption
    });
  }

  function testDepositPrivate() public returns (bytes32) {
    // mint token and approve to the portal
    portalERC20.mint(address(this), mintAmount);
    portalERC20.approve(address(tokenPortal), mintAmount);

    // Check for the expected message
    DataStructures.L1ToL2Msg memory expectedMessage = _createExpectedMintPrivateL1ToL2Message();

    bytes32 expectedLeaf = expectedMessage.sha256ToField();

    // Check the event was emitted
    vm.expectEmit(true, true, true, true);
    // event we expect
    emit IInbox.MessageSent(FIRST_REAL_TREE_NUM, 0, expectedLeaf);
    // event we will get

    // Perform op
    bytes32 leaf = tokenPortal.depositToAztecPrivate(
      secretHashForRedeemingMintedNotes, amount, secretHashForL2MessageConsumption
    );

    assertEq(leaf, expectedLeaf, "returned leaf and calculated leaf should match");

    return leaf;
  }

  function testDepositPublic() public returns (bytes32) {
    // mint token and approve to the portal
    portalERC20.mint(address(this), mintAmount);
    portalERC20.approve(address(tokenPortal), mintAmount);

    // Check for the expected message
    DataStructures.L1ToL2Msg memory expectedMessage = _createExpectedMintPublicL1ToL2Message();
    bytes32 expectedLeaf = expectedMessage.sha256ToField();

    // Check the event was emitted
    vm.expectEmit(true, true, true, true);
    // event we expect
    emit IInbox.MessageSent(FIRST_REAL_TREE_NUM, 0, expectedLeaf);

    // Perform op
    bytes32 leaf = tokenPortal.depositToAztecPublic(to, amount, secretHashForL2MessageConsumption);

    assertEq(leaf, expectedLeaf, "returned leaf and calculated leaf should match");

    return leaf;
  }

  function _createWithdrawMessageForOutbox(address _designatedCaller)
    internal
    returns (bytes32, bytes32)
  {
    bytes32 l2ToL1Message = Hash.sha256ToField(
      DataStructures.L2ToL1Msg({
        sender: DataStructures.L2Actor({actor: l2TokenAddress, version: 1}),
        recipient: DataStructures.L1Actor({actor: address(tokenPortal), chainId: block.chainid}),
        content: Hash.sha256ToField(
          abi.encodeWithSignature(
            "withdraw(address,uint256,address)", recipient, withdrawAmount, _designatedCaller
          )
          )
      })
    );

    uint256 treeHeight = 1;
    NaiveMerkle tree = new NaiveMerkle(treeHeight);
    tree.insertLeaf(l2ToL1Message);
    bytes32 treeRoot = tree.computeRoot();

    return (l2ToL1Message, treeRoot);
  }

  function _addWithdrawMessageInOutbox(address _designatedCaller, uint256 _l2BlockNumber)
    internal
    returns (bytes32, bytes32[] memory, bytes32)
  {
    // send assets to the portal
    portalERC20.mint(address(tokenPortal), withdrawAmount);

    // Create the message
    (bytes32 l2ToL1Message,) = _createWithdrawMessageForOutbox(_designatedCaller);

    uint256 treeHeight = 1;
    NaiveMerkle tree = new NaiveMerkle(treeHeight);
    tree.insertLeaf(l2ToL1Message);

    (bytes32[] memory siblingPath,) = tree.computeSiblingPath(0);

    bytes32 treeRoot = tree.computeRoot();
    // Insert messages into the outbox (impersonating the rollup contract)
    vm.prank(address(rollup));
    outbox.insert(_l2BlockNumber, treeRoot, treeHeight);

    return (l2ToL1Message, siblingPath, treeRoot);
  }

  function testAnyoneCanCallWithdrawIfNoDesignatedCaller(address _caller) public {
    vm.assume(_caller != address(0));

    uint256 l2BlockNumber = 69;
    // add message with caller as this address
    (bytes32 l2ToL1Message, bytes32[] memory siblingPath, bytes32 treeRoot) =
      _addWithdrawMessageInOutbox(address(0), l2BlockNumber);
    assertEq(portalERC20.balanceOf(recipient), 0);

    vm.startPrank(_caller);
    vm.expectEmit(true, true, true, true);
    emit IOutbox.MessageConsumed(l2BlockNumber, treeRoot, l2ToL1Message, 0);
    tokenPortal.withdraw(recipient, withdrawAmount, false, l2BlockNumber, 0, siblingPath);

    // Should have received 654 RNA tokens
    assertEq(portalERC20.balanceOf(recipient), withdrawAmount);

    // Should not be able to withdraw again
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Outbox__AlreadyNullified.selector, l2BlockNumber, 0)
    );
    tokenPortal.withdraw(recipient, withdrawAmount, false, l2BlockNumber, 0, siblingPath);
    vm.stopPrank();
  }

  function testWithdrawWithDesignatedCallerFailsForOtherCallers(address _caller) public {
    vm.assume(_caller != address(this));
    uint256 l2BlockNumber = 69;
    // add message with caller as this address
    (, bytes32[] memory siblingPath, bytes32 treeRoot) =
      _addWithdrawMessageInOutbox(address(this), l2BlockNumber);

    vm.startPrank(_caller);
    (bytes32 l2ToL1MessageHash, bytes32 consumedRoot) = _createWithdrawMessageForOutbox(_caller);
    vm.expectRevert(
      abi.encodeWithSelector(
        Errors.MerkleLib__InvalidRoot.selector, treeRoot, consumedRoot, l2ToL1MessageHash, 0
      )
    );
    tokenPortal.withdraw(recipient, withdrawAmount, true, l2BlockNumber, 0, siblingPath);

    (l2ToL1MessageHash, consumedRoot) = _createWithdrawMessageForOutbox(address(0));
    vm.expectRevert(
      abi.encodeWithSelector(
        Errors.MerkleLib__InvalidRoot.selector, treeRoot, consumedRoot, l2ToL1MessageHash, 0
      )
    );
    tokenPortal.withdraw(recipient, withdrawAmount, false, l2BlockNumber, 0, siblingPath);
    vm.stopPrank();
  }

  function testWithdrawWithDesignatedCallerSucceedsForDesignatedCaller() public {
    uint256 l2BlockNumber = 69;
    // add message with caller as this address
    (bytes32 l2ToL1Message, bytes32[] memory siblingPath, bytes32 treeRoot) =
      _addWithdrawMessageInOutbox(address(this), l2BlockNumber);

    vm.expectEmit(true, true, true, true);
    emit IOutbox.MessageConsumed(l2BlockNumber, treeRoot, l2ToL1Message, 0);
    tokenPortal.withdraw(recipient, withdrawAmount, true, l2BlockNumber, 0, siblingPath);

    // Should have received 654 RNA tokens
    assertEq(portalERC20.balanceOf(recipient), withdrawAmount);
  }
}

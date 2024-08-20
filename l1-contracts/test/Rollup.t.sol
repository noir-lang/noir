// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DecoderBase} from "./decoders/Base.sol";

import {DataStructures} from "../src/core/libraries/DataStructures.sol";
import {Constants} from "../src/core/libraries/ConstantsGen.sol";

import {Registry} from "../src/core/messagebridge/Registry.sol";
import {Inbox} from "../src/core/messagebridge/Inbox.sol";
import {Outbox} from "../src/core/messagebridge/Outbox.sol";
import {Errors} from "../src/core/libraries/Errors.sol";
import {Rollup} from "../src/core/Rollup.sol";
import {IFeeJuicePortal} from "../src/core/interfaces/IFeeJuicePortal.sol";
import {FeeJuicePortal} from "../src/core/FeeJuicePortal.sol";
import {Leonidas} from "../src/core/sequencer_selection/Leonidas.sol";
import {AvailabilityOracle} from "../src/core/availability_oracle/AvailabilityOracle.sol";
import {FrontierMerkle} from "../src/core/messagebridge/frontier_tree/Frontier.sol";
import {NaiveMerkle} from "./merkle/Naive.sol";
import {MerkleTestUtil} from "./merkle/TestUtil.sol";
import {PortalERC20} from "./portals/PortalERC20.sol";

import {TxsDecoderHelper} from "./decoders/helpers/TxsDecoderHelper.sol";
import {IERC20Errors} from "@oz/interfaces/draft-IERC6093.sol";

/**
 * Blocks are generated using the `integration_l1_publisher.test.ts` tests.
 * Main use of these test is shorter cycles when updating the decoder contract.
 */
contract RollupTest is DecoderBase {
  Registry internal registry;
  Inbox internal inbox;
  Outbox internal outbox;
  Rollup internal rollup;
  MerkleTestUtil internal merkleTestUtil;
  TxsDecoderHelper internal txsHelper;
  PortalERC20 internal portalERC20;
  FeeJuicePortal internal feeJuicePortal;

  AvailabilityOracle internal availabilityOracle;

  /**
   * @notice  Set up the contracts needed for the tests with time aligned to the provided block name
   */
  modifier setUpFor(string memory _name) {
    {
      Leonidas leo = new Leonidas(address(1));
      DecoderBase.Full memory full = load(_name);
      uint256 slotNumber = full.block.decodedHeader.globalVariables.slotNumber;
      uint256 initialTime =
        full.block.decodedHeader.globalVariables.timestamp - slotNumber * leo.SLOT_DURATION();
      vm.warp(initialTime);
    }

    registry = new Registry(address(this));
    availabilityOracle = new AvailabilityOracle();
    portalERC20 = new PortalERC20();
    feeJuicePortal = new FeeJuicePortal(address(this));
    portalERC20.mint(address(feeJuicePortal), Constants.FEE_JUICE_INITIAL_MINT);
    feeJuicePortal.initialize(
      address(registry), address(portalERC20), bytes32(Constants.FEE_JUICE_ADDRESS)
    );
    rollup = new Rollup(
      registry,
      availabilityOracle,
      IFeeJuicePortal(address(feeJuicePortal)),
      bytes32(0),
      address(this)
    );
    inbox = Inbox(address(rollup.INBOX()));
    outbox = Outbox(address(rollup.OUTBOX()));

    registry.upgrade(address(rollup));

    merkleTestUtil = new MerkleTestUtil();
    txsHelper = new TxsDecoderHelper();
    _;
  }

  function testRevertPrune() public setUpFor("mixed_block_1") {
    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__NothingToPrune.selector));
    rollup.prune();

    _testBlock("mixed_block_1", false);

    uint256 currentSlot = rollup.getCurrentSlot();
    (, uint128 slot,) = rollup.blocks(1);
    uint256 prunableAt = uint256(slot) + rollup.TIMELINESS_PROVING_IN_SLOTS();

    vm.expectRevert(
      abi.encodeWithSelector(Errors.Rollup__NotReadyToPrune.selector, currentSlot, prunableAt)
    );
    rollup.prune();
  }

  function testPrune() public setUpFor("mixed_block_1") {
    _testBlock("mixed_block_1", false);

    assertEq(inbox.inProgress(), 3, "Invalid in progress");

    // @note  Fetch the inbox root of block 2. This should be frozen when block 1 is proposed.
    //        Even if we end up reverting block 1, we should still see the same root in the inbox.
    bytes32 inboxRoot2 = inbox.trees(2).root();

    (, uint128 slot,) = rollup.blocks(1);
    uint256 prunableAt = uint256(slot) + rollup.TIMELINESS_PROVING_IN_SLOTS();

    uint256 timeOfPrune = rollup.getTimestampForSlot(prunableAt);
    vm.warp(timeOfPrune);

    assertEq(rollup.pendingBlockCount(), 2, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1, "Invalid proven block count");

    // @note  Get the root and min height that we have in the outbox.
    //        We read it directly in storage because it is not yet proven, so the getter will give (0, 0).
    //        The values are stored such that we can check that after pruning, and inserting a new block,
    //        we will override it.
    bytes32 rootMixed = vm.load(address(outbox), keccak256(abi.encode(1, 0)));
    uint256 minHeightMixed =
      uint256(vm.load(address(outbox), bytes32(uint256(keccak256(abi.encode(1, 0))) + 1)));

    assertNotEq(rootMixed, bytes32(0), "Invalid root");
    assertNotEq(minHeightMixed, 0, "Invalid min height");

    rollup.prune();
    assertEq(inbox.inProgress(), 3, "Invalid in progress");
    assertEq(rollup.pendingBlockCount(), 1, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1, "Invalid proven block count");

    // @note  We alter what slot is specified in the empty block!
    //        This means that we keep the `empty_block_1` mostly as is, but replace the slot number
    //        and timestamp as if it was created at a different point in time. This allow us to insert it
    //        as if it was the first block, even after we had originally inserted the mixed block.
    //        An example where this could happen would be if no-one could proof the mixed block.
    _testBlock("empty_block_1", false, prunableAt);

    assertEq(inbox.inProgress(), 3, "Invalid in progress");
    assertEq(inbox.trees(2).root(), inboxRoot2, "Invalid inbox root");
    assertEq(rollup.pendingBlockCount(), 2, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1, "Invalid proven block count");

    // We check that the roots in the outbox have correctly been updated.
    bytes32 rootEmpty = vm.load(address(outbox), keccak256(abi.encode(1, 0)));
    uint256 minHeightEmpty =
      uint256(vm.load(address(outbox), bytes32(uint256(keccak256(abi.encode(1, 0))) + 1)));

    assertNotEq(rootEmpty, bytes32(0), "Invalid root");
    assertNotEq(minHeightEmpty, 0, "Invalid min height");
    assertNotEq(rootEmpty, rootMixed, "Invalid root");
    assertNotEq(minHeightEmpty, minHeightMixed, "Invalid min height");
  }

  function testBlockFee() public setUpFor("mixed_block_1") {
    uint256 feeAmount = 2e18;

    DecoderBase.Data memory data = load("mixed_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;
    bytes memory body = data.body;

    assembly {
      mstore(add(header, add(0x20, 0x0248)), feeAmount)
    }
    availabilityOracle.publish(body);

    assertEq(portalERC20.balanceOf(address(rollup)), 0, "invalid rollup balance");

    uint256 portalBalance = portalERC20.balanceOf(address(feeJuicePortal));

    vm.expectRevert(
      abi.encodeWithSelector(
        IERC20Errors.ERC20InsufficientBalance.selector,
        address(feeJuicePortal),
        portalBalance,
        feeAmount
      )
    );
    rollup.process(header, archive);

    address coinbase = data.decodedHeader.globalVariables.coinbase;
    uint256 coinbaseBalance = portalERC20.balanceOf(coinbase);
    assertEq(coinbaseBalance, 0, "invalid initial coinbase balance");

    portalERC20.mint(address(feeJuicePortal), feeAmount - portalBalance);

    rollup.process(header, archive);
    assertEq(portalERC20.balanceOf(coinbase), feeAmount, "invalid coinbase balance");
  }

  function testMixedBlock(bool _toProve) public setUpFor("mixed_block_1") {
    _testBlock("mixed_block_1", _toProve);

    assertEq(rollup.pendingBlockCount(), 2, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), _toProve ? 2 : 1, "Invalid proven block count");
  }

  function testConsecutiveMixedBlocks(uint256 _blocksToProve) public setUpFor("mixed_block_1") {
    uint256 toProve = bound(_blocksToProve, 0, 2);

    _testBlock("mixed_block_1", toProve > 0);
    _testBlock("mixed_block_2", toProve > 1);

    assertEq(rollup.pendingBlockCount(), 3, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1 + toProve, "Invalid proven block count");
  }

  function testConsecutiveMixedBlocksNonSequentialProof() public setUpFor("mixed_block_1") {
    _testBlock("mixed_block_1", false);
    _testBlock("mixed_block_2", true);

    assertTrue(rollup.isBlockProven(2), "Block 2 is not proven");

    assertEq(rollup.pendingBlockCount(), 3, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1, "Invalid proven block count");
  }

  function testEmptyBlock(bool _toProve) public setUpFor("empty_block_1") {
    _testBlock("empty_block_1", _toProve);
    assertEq(rollup.pendingBlockCount(), 2, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), _toProve ? 2 : 1, "Invalid proven block count");
  }

  function testConsecutiveEmptyBlocks(uint256 _blocksToProve) public setUpFor("empty_block_1") {
    uint256 toProve = bound(_blocksToProve, 0, 2);
    _testBlock("empty_block_1", toProve > 0);
    _testBlock("empty_block_2", toProve > 1);

    assertEq(rollup.pendingBlockCount(), 3, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1 + toProve, "Invalid proven block count");
  }

  function testRevertInvalidBlockNumber() public setUpFor("empty_block_1") {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;
    bytes memory body = data.body;

    assembly {
      // TODO: Hardcoding offsets in the middle of tests is annoying to say the least.
      mstore(add(header, add(0x20, 0x0174)), 0x420)
    }

    availabilityOracle.publish(body);

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__InvalidBlockNumber.selector, 1, 0x420));
    rollup.process(header, archive);
  }

  function testRevertInvalidChainId() public setUpFor("empty_block_1") {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;
    bytes memory body = data.body;

    assembly {
      // TODO: Hardcoding offsets in the middle of tests is annoying to say the least.
      mstore(add(header, add(0x20, 0x0134)), 0x420)
    }

    availabilityOracle.publish(body);

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__InvalidChainId.selector, 31337, 0x420));
    rollup.process(header, archive);
  }

  function testRevertInvalidVersion() public setUpFor("empty_block_1") {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;
    bytes memory body = data.body;

    assembly {
      mstore(add(header, add(0x20, 0x0154)), 0x420)
    }

    availabilityOracle.publish(body);

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__InvalidVersion.selector, 1, 0x420));
    rollup.process(header, archive);
  }

  function testRevertInvalidTimestamp() public setUpFor("empty_block_1") {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;
    bytes memory body = data.body;

    uint256 realTs = data.decodedHeader.globalVariables.timestamp;
    uint256 badTs = realTs + 1;

    vm.warp(max(block.timestamp, realTs));

    assembly {
      mstore(add(header, add(0x20, 0x01b4)), badTs)
    }

    availabilityOracle.publish(body);

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__InvalidTimestamp.selector, realTs, badTs));
    rollup.process(header, archive);
  }

  function testBlocksWithAssumeProven() public setUpFor("mixed_block_1") {
    rollup.setAssumeProvenUntilBlockNumber(2);
    _testBlock("mixed_block_1", false);
    _testBlock("mixed_block_2", false);

    assertEq(rollup.pendingBlockCount(), 3, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 2, "Invalid proven block count");
  }

  function testSetAssumeProvenAfterBlocksProcessed() public setUpFor("mixed_block_1") {
    _testBlock("mixed_block_1", false);
    _testBlock("mixed_block_2", false);
    rollup.setAssumeProvenUntilBlockNumber(2);

    assertEq(rollup.pendingBlockCount(), 3, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 2, "Invalid proven block count");
  }

  function testSubmitProofNonExistantBlock() public setUpFor("empty_block_1") {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__TryingToProveNonExistingBlock.selector));
    rollup.submitProof(header, archive, bytes32(0), "", "");
  }

  function testSubmitProofInvalidArchive() public setUpFor("empty_block_1") {
    _testBlock("empty_block_1", false);
    _testBlock("empty_block_2", false);

    DecoderBase.Data memory data = load("empty_block_2").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;

    // Update the lastArchive value in the header and then submit a proof
    assembly {
      mstore(add(header, add(0x20, 0x00)), 0xdeadbeef)
    }

    vm.expectRevert(
      abi.encodeWithSelector(
        Errors.Rollup__InvalidArchive.selector, rollup.archiveAt(1), 0xdeadbeef
      )
    );
    rollup.submitProof(header, archive, bytes32(0), "", "");
  }

  function testSubmitProofInvalidProposedArchive() public setUpFor("empty_block_1") {
    _testBlock("empty_block_1", false);

    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;

    bytes32 badArchive = keccak256(abi.encode(archive));

    vm.expectRevert(
      abi.encodeWithSelector(Errors.Rollup__InvalidProposedArchive.selector, archive, badArchive)
    );
    rollup.submitProof(header, badArchive, bytes32(0), "", "");
  }

  function _testBlock(string memory name, bool _submitProof) public {
    _testBlock(name, _submitProof, 0);
  }

  function _testBlock(string memory name, bool _submitProof, uint256 _slotNumber) public {
    DecoderBase.Full memory full = load(name);
    bytes memory header = full.block.header;
    bytes32 archive = full.block.archive;
    bytes memory body = full.block.body;
    uint32 numTxs = full.block.numTxs;

    // Overwrite some timestamps if needed
    if (_slotNumber != 0) {
      uint256 ts = rollup.getTimestampForSlot(_slotNumber);

      full.block.decodedHeader.globalVariables.timestamp = ts;
      full.block.decodedHeader.globalVariables.slotNumber = _slotNumber;
      assembly {
        mstore(add(header, add(0x20, 0x0194)), _slotNumber)
        mstore(add(header, add(0x20, 0x01b4)), ts)
      }
    }

    // We jump to the time of the block. (unless it is in the past)
    vm.warp(max(block.timestamp, full.block.decodedHeader.globalVariables.timestamp));

    _populateInbox(full.populate.sender, full.populate.recipient, full.populate.l1ToL2Content);

    availabilityOracle.publish(body);

    rollup.process(header, archive);

    if (_submitProof) {
      rollup.submitProof(header, archive, bytes32(0), "", "");

      assertTrue(
        rollup.isBlockProven(full.block.decodedHeader.globalVariables.blockNumber),
        "Block not proven"
      );
    }

    bytes32 l2ToL1MessageTreeRoot;
    {
      // NB: The below works with full blocks because we require the largest possible subtrees
      // for L2 to L1 messages - usually we make variable height subtrees, the roots of which
      // form a balanced tree

      // The below is a little janky - we know that this test deals with full txs with equal numbers
      // of msgs or txs with no messages, so the division works
      // TODO edit full.messages to include information about msgs per tx?
      uint256 subTreeHeight = full.messages.l2ToL1Messages.length == 0
        ? 0
        : merkleTestUtil.calculateTreeHeightFromSize(full.messages.l2ToL1Messages.length / numTxs);
      uint256 outHashTreeHeight = merkleTestUtil.calculateTreeHeightFromSize(numTxs);
      uint256 numMessagesWithPadding = numTxs * Constants.MAX_L2_TO_L1_MSGS_PER_TX;

      uint256 treeHeight = subTreeHeight + outHashTreeHeight;
      NaiveMerkle tree = new NaiveMerkle(treeHeight);
      for (uint256 i = 0; i < numMessagesWithPadding; i++) {
        if (i < full.messages.l2ToL1Messages.length) {
          tree.insertLeaf(full.messages.l2ToL1Messages[i]);
        } else {
          tree.insertLeaf(bytes32(0));
        }
      }

      l2ToL1MessageTreeRoot = tree.computeRoot();
    }

    (bytes32 root,) = outbox.getRootData(full.block.decodedHeader.globalVariables.blockNumber);

    // If we are trying to read a block beyond the proven chain, we should see "nothing".
    if (rollup.provenBlockCount() > full.block.decodedHeader.globalVariables.blockNumber) {
      assertEq(l2ToL1MessageTreeRoot, root, "Invalid l2 to l1 message tree root");
    } else {
      assertEq(root, bytes32(0), "Invalid outbox root");
    }

    assertEq(rollup.archive(), archive, "Invalid archive");
  }

  function _populateInbox(address _sender, bytes32 _recipient, bytes32[] memory _contents) internal {
    for (uint256 i = 0; i < _contents.length; i++) {
      vm.prank(_sender);
      inbox.sendL2Message(
        DataStructures.L2Actor({actor: _recipient, version: 1}), _contents[i], bytes32(0)
      );
    }
  }
}

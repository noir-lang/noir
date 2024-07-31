// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {IERC20} from "@oz/token/ERC20/IERC20.sol";

import {DecoderBase} from "./decoders/Base.sol";

import {DataStructures} from "../src/core/libraries/DataStructures.sol";
import {Constants} from "../src/core/libraries/ConstantsGen.sol";

import {Registry} from "../src/core/messagebridge/Registry.sol";
import {Inbox} from "../src/core/messagebridge/Inbox.sol";
import {Outbox} from "../src/core/messagebridge/Outbox.sol";
import {Errors} from "../src/core/libraries/Errors.sol";
import {Rollup} from "../src/core/Rollup.sol";
import {AvailabilityOracle} from "../src/core/availability_oracle/AvailabilityOracle.sol";
import {NaiveMerkle} from "./merkle/Naive.sol";
import {MerkleTestUtil} from "./merkle/TestUtil.sol";
import {PortalERC20} from "./portals/PortalERC20.sol";

import {TxsDecoderHelper} from "./decoders/helpers/TxsDecoderHelper.sol";

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

  AvailabilityOracle internal availabilityOracle;

  function setUp() public virtual {
    registry = new Registry();
    availabilityOracle = new AvailabilityOracle();
    portalERC20 = new PortalERC20();
    rollup = new Rollup(registry, availabilityOracle, IERC20(address(portalERC20)), bytes32(0));
    inbox = Inbox(address(rollup.INBOX()));
    outbox = Outbox(address(rollup.OUTBOX()));

    registry.upgrade(address(rollup), address(inbox), address(outbox));

    // mint some tokens to the rollup
    portalERC20.mint(address(rollup), 1000000);

    merkleTestUtil = new MerkleTestUtil();
    txsHelper = new TxsDecoderHelper();
  }

  function testMixedBlock(bool _toProve) public {
    _testBlock("mixed_block_1", _toProve);

    assertEq(rollup.pendingBlockCount(), 2, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), _toProve ? 2 : 1, "Invalid proven block count");
  }

  function testConsecutiveMixedBlocks(uint256 _blocksToProve) public {
    uint256 toProve = bound(_blocksToProve, 0, 2);

    _testBlock("mixed_block_1", toProve > 0);
    _testBlock("mixed_block_2", toProve > 1);

    assertEq(rollup.pendingBlockCount(), 3, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1 + toProve, "Invalid proven block count");
  }

  function testConsecutiveMixedBlocksNonSequentialProof() public {
    _testBlock("mixed_block_1", false);
    _testBlock("mixed_block_2", true);

    assertTrue(rollup.isBlockProven(2), "Block 2 is not proven");

    assertEq(rollup.pendingBlockCount(), 3, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1, "Invalid proven block count");
  }

  function testEmptyBlock(bool _toProve) public {
    _testBlock("empty_block_1", _toProve);
    assertEq(rollup.pendingBlockCount(), 2, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), _toProve ? 2 : 1, "Invalid proven block count");
  }

  function testConsecutiveEmptyBlocks(uint256 _blocksToProve) public {
    uint256 toProve = bound(_blocksToProve, 0, 2);
    _testBlock("empty_block_1", toProve > 0);
    _testBlock("empty_block_2", toProve > 1);

    assertEq(rollup.pendingBlockCount(), 3, "Invalid pending block count");
    assertEq(rollup.provenBlockCount(), 1 + toProve, "Invalid proven block count");
  }

  function testRevertInvalidBlockNumber() public {
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

  function testRevertInvalidChainId() public {
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

  function testRevertInvalidVersion() public {
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

  function testRevertTimestampInFuture() public {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;
    bytes memory body = data.body;

    uint256 ts = block.timestamp + 1;
    assembly {
      mstore(add(header, add(0x20, 0x0194)), ts)
    }

    availabilityOracle.publish(body);

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__TimestampInFuture.selector));
    rollup.process(header, archive);
  }

  function testRevertTimestampTooOld() public {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;
    bytes memory body = data.body;

    // Beware of the store slot below, if the test is failing might be because of the slot
    // We overwrite `lastBlockTs` in the rollup
    vm.store(address(rollup), bytes32(uint256(6)), bytes32(uint256(block.timestamp)));

    availabilityOracle.publish(body);

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__TimestampTooOld.selector));
    rollup.process(header, archive);
  }

  function testSubmitProofNonExistantBlock() public {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;

    vm.expectRevert(abi.encodeWithSelector(Errors.Rollup__TryingToProveNonExistingBlock.selector));
    rollup.submitProof(header, archive, bytes32(0), "", "");
  }

  function testSubmitProofInvalidArchive() public {
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

  function testSubmitProofInvalidProposedArchive() public {
    DecoderBase.Data memory data = load("empty_block_1").block;
    bytes memory header = data.header;
    bytes32 archive = data.archive;
    bytes memory body = data.body;

    availabilityOracle.publish(body);
    rollup.process(header, archive);

    bytes32 badArchive = keccak256(abi.encode(archive));

    vm.expectRevert(
      abi.encodeWithSelector(Errors.Rollup__InvalidProposedArchive.selector, archive, badArchive)
    );
    rollup.submitProof(header, badArchive, bytes32(0), "", "");
  }

  function _testBlock(string memory name, bool _submitProof) public {
    DecoderBase.Full memory full = load(name);
    bytes memory header = full.block.header;
    bytes32 archive = full.block.archive;
    bytes memory body = full.block.body;
    uint32 numTxs = full.block.numTxs;

    // We jump to the time of the block. (unless it is in the past)
    vm.warp(max(block.timestamp, full.block.decodedHeader.globalVariables.timestamp));

    _populateInbox(full.populate.sender, full.populate.recipient, full.populate.l1ToL2Content);

    availabilityOracle.publish(body);

    uint256 toConsume = inbox.toConsume();

    rollup.process(header, archive);

    if (_submitProof) {
      rollup.submitProof(header, archive, bytes32(0), "", "");

      assertTrue(
        rollup.isBlockProven(full.block.decodedHeader.globalVariables.blockNumber),
        "Block not proven"
      );
    }

    assertEq(inbox.toConsume(), toConsume + 1, "Message subtree not consumed");

    bytes32 l2ToL1MessageTreeRoot;
    {
      // NB: The below works with full blocks because we require the largest possible subtrees
      // for L2 to L1 messages - usually we make variable height subtrees, the roots of which
      // form a balanced tree

      // The below is a little janky - we know that this test deals with full txs with equal numbers
      // of msgs or txs with no messages, so the division works
      // TODO edit full.messages to include information about msgs per tx?
      uint256 subTreeHeight = merkleTestUtil.calculateTreeHeightFromSize(
        full.messages.l2ToL1Messages.length == 0 ? 0 : full.messages.l2ToL1Messages.length / numTxs
      );
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

    (bytes32 root,) = outbox.roots(full.block.decodedHeader.globalVariables.blockNumber);

    assertEq(l2ToL1MessageTreeRoot, root, "Invalid l2 to l1 message tree root");

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

  function max(uint256 a, uint256 b) internal pure returns (uint256) {
    return a > b ? a : b;
  }
}

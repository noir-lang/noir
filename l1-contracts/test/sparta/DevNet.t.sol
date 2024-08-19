// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DecoderBase} from "../decoders/Base.sol";

import {DataStructures} from "../../src/core/libraries/DataStructures.sol";
import {Constants} from "../../src/core/libraries/ConstantsGen.sol";
import {SignatureLib} from "../../src/core/sequencer_selection/SignatureLib.sol";

import {Registry} from "../../src/core/messagebridge/Registry.sol";
import {Inbox} from "../../src/core/messagebridge/Inbox.sol";
import {Outbox} from "../../src/core/messagebridge/Outbox.sol";
import {Errors} from "../../src/core/libraries/Errors.sol";
import {Rollup} from "../../src/core/Rollup.sol";
import {Leonidas} from "../../src/core/sequencer_selection/Leonidas.sol";
import {AvailabilityOracle} from "../../src/core/availability_oracle/AvailabilityOracle.sol";
import {NaiveMerkle} from "../merkle/Naive.sol";
import {MerkleTestUtil} from "../merkle/TestUtil.sol";
import {TxsDecoderHelper} from "../decoders/helpers/TxsDecoderHelper.sol";
import {IFeeJuicePortal} from "../../src/core/interfaces/IFeeJuicePortal.sol";

/**
 * We are using the same blocks as from Rollup.t.sol.
 * The tests in this file is testing the sequencer selection
 *
 * We will skip these test if we are running with IS_DEV_NET = true
 */
contract DevNetTest is DecoderBase {
  Registry internal registry;
  Inbox internal inbox;
  Outbox internal outbox;
  Rollup internal rollup;
  MerkleTestUtil internal merkleTestUtil;
  TxsDecoderHelper internal txsHelper;

  AvailabilityOracle internal availabilityOracle;

  mapping(address validator => uint256 privateKey) internal privateKeys;

  SignatureLib.Signature internal emptySignature;

  /**
   * @notice  Set up the contracts needed for the tests with time aligned to the provided block name
   */
  modifier setup(uint256 _validatorCount) {
    string memory _name = "mixed_block_1";
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
    rollup = new Rollup(
      registry, availabilityOracle, IFeeJuicePortal(address(0)), bytes32(0), address(this)
    );
    inbox = Inbox(address(rollup.INBOX()));
    outbox = Outbox(address(rollup.OUTBOX()));

    registry.upgrade(address(rollup));

    merkleTestUtil = new MerkleTestUtil();
    txsHelper = new TxsDecoderHelper();

    for (uint256 i = 1; i < _validatorCount + 1; i++) {
      uint256 privateKey = uint256(keccak256(abi.encode("validator", i)));
      address validator = vm.addr(privateKey);
      privateKeys[validator] = privateKey;
      rollup.addValidator(validator);
    }
    _;
  }

  function testProposerForNonSetupEpoch(uint8 _epochsToJump) public setup(5) {
    if (Constants.IS_DEV_NET == 0) {
      return;
    }

    uint256 pre = rollup.getCurrentEpoch();
    vm.warp(
      block.timestamp + uint256(_epochsToJump) * rollup.EPOCH_DURATION() * rollup.SLOT_DURATION()
    );
    uint256 post = rollup.getCurrentEpoch();
    assertEq(pre + _epochsToJump, post, "Invalid epoch");

    address expectedProposer = rollup.getCurrentProposer();

    // Add a validator which will also setup the epoch
    rollup.addValidator(address(0xdead));

    address actualProposer = rollup.getCurrentProposer();
    assertEq(expectedProposer, actualProposer, "Invalid proposer");
  }

  function testNoValidators() public setup(0) {
    if (Constants.IS_DEV_NET == 0) {
      return;
    }

    _testBlock("mixed_block_1", false, false);
  }

  function testInvalidProposer() public setup(1) {
    if (Constants.IS_DEV_NET == 0) {
      return;
    }

    _testBlock("mixed_block_1", true, true);
  }

  struct StructToAvoidDeepStacks {
    uint256 needed;
    address proposer;
    bool shouldRevert;
  }

  function _testBlock(string memory _name, bool _expectRevert, bool _invalidProposer) internal {
    _testBlock(_name, _expectRevert, _invalidProposer, 0);
  }

  function _testBlock(string memory _name, bool _expectRevert, bool _invalidProposer, uint256 _ts)
    internal
  {
    DecoderBase.Full memory full = load(_name);
    bytes memory header = full.block.header;
    bytes32 archive = full.block.archive;
    bytes memory body = full.block.body;

    StructToAvoidDeepStacks memory ree;

    // We jump to the time of the block. (unless it is in the past)
    vm.warp(max(block.timestamp, max(full.block.decodedHeader.globalVariables.timestamp, _ts)));

    if (_ts > 0) {
      // Update the timestamp and slot in the header
      uint256 slotValue = rollup.getCurrentSlot();
      uint256 timestampMemoryPosition = 0x01b4;
      uint256 slotMemoryPosition = 0x0194;
      assembly {
        mstore(add(header, add(0x20, timestampMemoryPosition)), _ts)
        mstore(add(header, add(0x20, slotMemoryPosition)), slotValue)
      }
    }

    _populateInbox(full.populate.sender, full.populate.recipient, full.populate.l1ToL2Content);

    availabilityOracle.publish(body);

    ree.proposer = rollup.getCurrentProposer();
    ree.shouldRevert = false;

    rollup.setupEpoch();

    if (_invalidProposer) {
      ree.proposer = address(uint160(uint256(keccak256(abi.encode("invalid", ree.proposer)))));
      // Why don't we end up here?
      vm.expectRevert(
        abi.encodeWithSelector(Errors.Leonidas__InvalidProposer.selector, address(0), ree.proposer)
      );
      ree.shouldRevert = true;
    }

    vm.prank(ree.proposer);
    rollup.process(header, archive);

    assertEq(_expectRevert, ree.shouldRevert, "Invalid revert expectation");

    if (ree.shouldRevert) {
      return;
    }

    bytes32 l2ToL1MessageTreeRoot;
    {
      uint32 numTxs = full.block.numTxs;
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

    (bytes32 root,) = outbox.getRootData(full.block.decodedHeader.globalVariables.blockNumber);

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

// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {DecoderBase} from "../decoders/Base.sol";

import {DataStructures} from "../../src/core/libraries/DataStructures.sol";
import {Constants} from "../../src/core/libraries/ConstantsGen.sol";
import {SignatureLib} from "../../src/core/sequencer_selection/SignatureLib.sol";
import {MessageHashUtils} from "@oz/utils/cryptography/MessageHashUtils.sol";

import {Registry} from "../../src/core/messagebridge/Registry.sol";
import {Inbox} from "../../src/core/messagebridge/Inbox.sol";
import {Outbox} from "../../src/core/messagebridge/Outbox.sol";
import {Errors} from "../../src/core/libraries/Errors.sol";
import {Rollup} from "../../src/core/Rollup.sol";
import {Leonidas} from "../../src/core/sequencer_selection/Leonidas.sol";
import {AvailabilityOracle} from "../../src/core/availability_oracle/AvailabilityOracle.sol";
import {NaiveMerkle} from "../merkle/Naive.sol";
import {MerkleTestUtil} from "../merkle/TestUtil.sol";
import {PortalERC20} from "../portals/PortalERC20.sol";
import {TxsDecoderHelper} from "../decoders/helpers/TxsDecoderHelper.sol";
import {IFeeJuicePortal} from "../../src/core/interfaces/IFeeJuicePortal.sol";
/**
 * We are using the same blocks as from Rollup.t.sol.
 * The tests in this file is testing the sequencer selection
 *
 * We will skip these test if we are running with IS_DEV_NET = true
 */

contract SpartaTest is DecoderBase {
  using MessageHashUtils for bytes32;

  Registry internal registry;
  Inbox internal inbox;
  Outbox internal outbox;
  Rollup internal rollup;
  MerkleTestUtil internal merkleTestUtil;
  TxsDecoderHelper internal txsHelper;
  PortalERC20 internal portalERC20;

  AvailabilityOracle internal availabilityOracle;

  mapping(address validator => uint256 privateKey) internal privateKeys;

  SignatureLib.Signature internal emptySignature;

  /**
   * @notice  Set up the contracts needed for the tests with time aligned to the provided block name
   */
  modifier setup(uint256 _validatorCount) {
    string memory _name = "mixed_block_1";
    {
      Leonidas leonidas = new Leonidas(address(1));
      DecoderBase.Full memory full = load(_name);
      uint256 slotNumber = full.block.decodedHeader.globalVariables.slotNumber;
      uint256 initialTime =
        full.block.decodedHeader.globalVariables.timestamp - slotNumber * leonidas.SLOT_DURATION();
      vm.warp(initialTime);
    }

    registry = new Registry(address(this));
    availabilityOracle = new AvailabilityOracle();
    portalERC20 = new PortalERC20();
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

  function testProposerForNonSetupEpoch(uint8 _epochsToJump) public setup(4) {
    if (Constants.IS_DEV_NET == 1) {
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

  function testValidatorSetLargerThanCommittee(bool _insufficientSigs) public setup(100) {
    if (Constants.IS_DEV_NET == 1) {
      return;
    }

    assertGt(rollup.getValidators().length, rollup.TARGET_COMMITTEE_SIZE(), "Not enough validators");
    _testBlock("mixed_block_1", false, 0, false); // We run a block before the epoch with validators

    uint256 ts = block.timestamp + rollup.EPOCH_DURATION() * rollup.SLOT_DURATION();

    uint256 committeSize = rollup.TARGET_COMMITTEE_SIZE() * 2 / 3 + (_insufficientSigs ? 0 : 1);
    _testBlock("mixed_block_2", _insufficientSigs, committeSize, false, ts); // We need signatures!

    assertEq(
      rollup.getEpochCommittee(rollup.getCurrentEpoch()).length,
      rollup.TARGET_COMMITTEE_SIZE(),
      "Invalid committee size"
    );
  }

  function testHappyPath() public setup(4) {
    if (Constants.IS_DEV_NET == 1) {
      return;
    }

    _testBlock("mixed_block_1", false, 0, false); // We run a block before the epoch with validators
    _testBlock("mixed_block_2", false, 3, false); // We need signatures!
  }

  function testInvalidProposer() public setup(4) {
    if (Constants.IS_DEV_NET == 1) {
      return;
    }

    _testBlock("mixed_block_1", false, 0, false); // We run a block before the epoch with validators
    _testBlock("mixed_block_2", true, 3, true); // We need signatures!
  }

  function testInsufficientSigs() public setup(4) {
    if (Constants.IS_DEV_NET == 1) {
      return;
    }

    _testBlock("mixed_block_1", false, 0, false); // We run a block before the epoch with validators
    _testBlock("mixed_block_2", true, 2, false); // We need signatures!
  }

  struct StructToAvoidDeepStacks {
    uint256 needed;
    address proposer;
    bool shouldRevert;
  }

  function _testBlock(
    string memory _name,
    bool _expectRevert,
    uint256 _signatureCount,
    bool _invalidaProposer
  ) internal {
    _testBlock(_name, _expectRevert, _signatureCount, _invalidaProposer, 0);
  }

  function _testBlock(
    string memory _name,
    bool _expectRevert,
    uint256 _signatureCount,
    bool _invalidaProposer,
    uint256 _ts
  ) internal {
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

    if (_signatureCount > 0 && ree.proposer != address(0)) {
      address[] memory validators = rollup.getEpochCommittee(rollup.getCurrentEpoch());
      ree.needed = validators.length * 2 / 3 + 1;

      SignatureLib.Signature[] memory signatures = new SignatureLib.Signature[](_signatureCount);

      for (uint256 i = 0; i < _signatureCount; i++) {
        signatures[i] = createSignature(validators[i], archive);
      }

      if (_expectRevert) {
        ree.shouldRevert = true;
        if (_signatureCount < ree.needed) {
          vm.expectRevert(
            abi.encodeWithSelector(
              Errors.Leonidas__InsufficientAttestationsProvided.selector,
              ree.needed,
              _signatureCount
            )
          );
        }
        // @todo Handle SignatureLib__InvalidSignature case
        // @todo Handle Leonidas__InsufficientAttestations case
      }

      if (_expectRevert && _invalidaProposer) {
        address realProposer = ree.proposer;
        ree.proposer = address(uint160(uint256(keccak256(abi.encode("invalid", ree.proposer)))));
        vm.expectRevert(
          abi.encodeWithSelector(
            Errors.Leonidas__InvalidProposer.selector, realProposer, ree.proposer
          )
        );
        ree.shouldRevert = true;
      }

      vm.prank(ree.proposer);
      rollup.process(header, archive, signatures);

      if (ree.shouldRevert) {
        return;
      }
    } else {
      rollup.process(header, archive);
    }

    assertEq(_expectRevert, ree.shouldRevert, "Invalid revert expectation");

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

  function createSignature(address _signer, bytes32 _digest)
    internal
    view
    returns (SignatureLib.Signature memory)
  {
    uint256 privateKey = privateKeys[_signer];
    bytes32 digestForSig = _digest.toEthSignedMessageHash();
    (uint8 v, bytes32 r, bytes32 s) = vm.sign(privateKey, digestForSig);

    return SignatureLib.Signature({isEmpty: false, v: v, r: r, s: s});
  }
}

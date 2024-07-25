// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {IERC20} from "@oz/token/ERC20/IERC20.sol";

import {DecoderBase} from "../decoders/Base.sol";

import {DataStructures} from "../../src/core/libraries/DataStructures.sol";
import {Constants} from "../../src/core/libraries/ConstantsGen.sol";
import {SignatureLib} from "../../src/core/sequencer_selection/SignatureLib.sol";

import {Registry} from "../../src/core/messagebridge/Registry.sol";
import {Inbox} from "../../src/core/messagebridge/Inbox.sol";
import {Outbox} from "../../src/core/messagebridge/Outbox.sol";
import {Errors} from "../../src/core/libraries/Errors.sol";
import {Rollup} from "../../src/core/Rollup.sol";
import {AvailabilityOracle} from "../../src/core/availability_oracle/AvailabilityOracle.sol";
import {NaiveMerkle} from "../merkle/Naive.sol";
import {MerkleTestUtil} from "../merkle/TestUtil.sol";
import {PortalERC20} from "../portals/PortalERC20.sol";
import {TxsDecoderHelper} from "../decoders/helpers/TxsDecoderHelper.sol";

/**
 * We are using the same blocks as from Rollup.t.sol.
 * The tests in this file is testing the sequencer selection
 */
contract SpartaTest is DecoderBase {
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

    for (uint256 i = 1; i < 5; i++) {
      uint256 privateKey = uint256(keccak256(abi.encode("validator", i)));
      address validator = vm.addr(privateKey);
      privateKeys[validator] = privateKey;
      rollup.addValidator(validator);
    }
  }

  function _testProposerForFutureEpoch() public {
    // @todo Implement
  }

  function _testValidatorSetLargerThanCommittee() public {
    // @todo Implement
  }

  function testHappyPath() public {
    _testBlock("mixed_block_0", 0, false); // We run a block before the epoch with validators
    _testBlock("mixed_block_1", 3, false); // We need signatures!
  }

  function testInvalidProposer() public {
    _testBlock("mixed_block_0", 0, false); // We run a block before the epoch with validators
    _testBlock("mixed_block_1", 3, true); // We need signatures!
  }

  function testInsufficientSigs() public {
    _testBlock("mixed_block_0", 0, false); // We run a block before the epoch with validators
    _testBlock("mixed_block_1", 2, false); // We need signatures!
  }

  struct StructToAvoidDeepStacks {
    uint256 needed;
    address proposer;
    bool shouldRevert;
  }

  function _testBlock(string memory _name, uint256 _signatureCount, bool _invalidaProposer) public {
    DecoderBase.Full memory full = load(_name);
    bytes memory header = full.block.header;
    bytes32 archive = full.block.archive;
    bytes memory body = full.block.body;

    StructToAvoidDeepStacks memory ree;

    // We jump to the time of the block. (unless it is in the past)
    vm.warp(max(block.timestamp, full.block.decodedHeader.globalVariables.timestamp));

    _populateInbox(full.populate.sender, full.populate.recipient, full.populate.l1ToL2Content);

    availabilityOracle.publish(body);

    uint256 toConsume = inbox.toConsume();
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

      if (_signatureCount < ree.needed) {
        vm.expectRevert(
          abi.encodeWithSelector(
            Errors.Leonidas__InsufficientAttestations.selector, ree.needed, _signatureCount
          )
        );
        ree.shouldRevert = true;
      }

      if (_invalidaProposer) {
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

    assertEq(inbox.toConsume(), toConsume + 1, "Message subtree not consumed");

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

  function createSignature(address _signer, bytes32 _digest)
    internal
    view
    returns (SignatureLib.Signature memory)
  {
    uint256 privateKey = privateKeys[_signer];
    (uint8 v, bytes32 r, bytes32 s) = vm.sign(privateKey, _digest);

    return SignatureLib.Signature({isEmpty: false, v: v, r: r, s: s});
  }
}

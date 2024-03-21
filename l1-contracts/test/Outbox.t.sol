// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";
import {Outbox} from "../src/core/messagebridge/Outbox.sol";
import {IOutbox} from "../src/core/interfaces/messagebridge/IOutbox.sol";
import {Errors} from "../src/core/libraries/Errors.sol";
import {DataStructures} from "../src/core/libraries/DataStructures.sol";
import {Hash} from "../src/core/libraries/Hash.sol";
import {NaiveMerkle} from "./merkle/Naive.sol";
import {MerkleTestUtil} from "./merkle/TestUtil.sol";

contract OutboxTest is Test {
  using Hash for DataStructures.L2ToL1Msg;

  address internal constant ROLLUP_CONTRACT = address(0x42069123);
  address internal constant NOT_RECIPIENT = address(0x420);
  uint256 internal constant DEFAULT_TREE_HEIGHT = 2;
  uint256 internal constant AZTEC_VERSION = 1;

  Outbox internal outbox;
  NaiveMerkle internal zeroedTree;
  MerkleTestUtil internal merkleTestUtil;

  function setUp() public {
    outbox = new Outbox(ROLLUP_CONTRACT);
    zeroedTree = new NaiveMerkle(DEFAULT_TREE_HEIGHT);
    merkleTestUtil = new MerkleTestUtil();
  }

  function _fakeMessage(address _recipient) internal view returns (DataStructures.L2ToL1Msg memory) {
    return DataStructures.L2ToL1Msg({
      sender: DataStructures.L2Actor({
        actor: 0x2000000000000000000000000000000000000000000000000000000000000000,
        version: AZTEC_VERSION
      }),
      recipient: DataStructures.L1Actor({actor: _recipient, chainId: block.chainid}),
      content: 0x3000000000000000000000000000000000000000000000000000000000000000
    });
  }

  function testRevertIfInsertingFromNonRollup(address _caller) public {
    vm.assume(ROLLUP_CONTRACT != _caller);
    bytes32 root = zeroedTree.computeRoot();

    vm.prank(_caller);
    vm.expectRevert(abi.encodeWithSelector(Errors.Outbox__Unauthorized.selector));
    outbox.insert(1, root, DEFAULT_TREE_HEIGHT);
  }

  function testRevertIfInsertingDuplicate() public {
    bytes32 root = zeroedTree.computeRoot();

    vm.prank(ROLLUP_CONTRACT);
    outbox.insert(1, root, DEFAULT_TREE_HEIGHT);

    vm.prank(ROLLUP_CONTRACT);
    vm.expectRevert(abi.encodeWithSelector(Errors.Outbox__RootAlreadySetAtBlock.selector, 1));
    outbox.insert(1, root, DEFAULT_TREE_HEIGHT);
  }

  // This function tests the insertion of random arrays of L2 to L1 messages
  // We make a naive tree with a computed height, insert the leafs into it, and compute a root. We then add the root as the root of the
  // L2 to L1 message tree, expect for the correct event to be emitted, and then query for the root in the contract—making sure the roots, as well as the
  // the tree height (which is also the length of the sibling path) match
  function testInsertVariedLeafs(bytes32[] calldata _messageLeafs) public {
    uint256 treeHeight = merkleTestUtil.calculateTreeHeightFromSize(_messageLeafs.length);
    NaiveMerkle tree = new NaiveMerkle(treeHeight);

    for (uint256 i = 0; i < _messageLeafs.length; i++) {
      vm.assume(_messageLeafs[i] != bytes32(0));
      tree.insertLeaf(_messageLeafs[i]);
    }

    bytes32 root = tree.computeRoot();

    vm.expectEmit(true, true, true, true, address(outbox));
    emit IOutbox.RootAdded(1, root, treeHeight);
    vm.prank(ROLLUP_CONTRACT);
    outbox.insert(1, root, treeHeight);

    (bytes32 actualRoot, uint256 actualHeight) = outbox.roots(1);
    assertEq(root, actualRoot);
    assertEq(treeHeight, actualHeight);
  }

  function testRevertIfConsumingMessageBelongingToOther() public {
    DataStructures.L2ToL1Msg memory fakeMessage = _fakeMessage(address(this));

    (bytes32[] memory path,) = zeroedTree.computeSiblingPath(0);

    vm.prank(NOT_RECIPIENT);
    vm.expectRevert(
      abi.encodeWithSelector(Errors.Outbox__InvalidRecipient.selector, address(this), NOT_RECIPIENT)
    );
    outbox.consume(fakeMessage, 1, 1, path);
  }

  function testRevertIfConsumingMessageWithInvalidChainId() public {
    DataStructures.L2ToL1Msg memory fakeMessage = _fakeMessage(address(this));

    (bytes32[] memory path,) = zeroedTree.computeSiblingPath(0);

    fakeMessage.recipient.chainId = block.chainid + 1;

    vm.expectRevert(abi.encodeWithSelector(Errors.Outbox__InvalidChainId.selector));
    outbox.consume(fakeMessage, 1, 1, path);
  }

  function testRevertIfNothingInsertedAtBlockNumber() public {
    uint256 blockNumber = 1;
    DataStructures.L2ToL1Msg memory fakeMessage = _fakeMessage(address(this));

    (bytes32[] memory path,) = zeroedTree.computeSiblingPath(0);

    vm.expectRevert(
      abi.encodeWithSelector(Errors.Outbox__NothingToConsumeAtBlock.selector, blockNumber)
    );
    outbox.consume(fakeMessage, blockNumber, 1, path);
  }

  function testRevertIfTryingToConsumeSameMessage() public {
    DataStructures.L2ToL1Msg memory fakeMessage = _fakeMessage(address(this));
    bytes32 leaf = fakeMessage.sha256ToField();

    NaiveMerkle tree = new NaiveMerkle(DEFAULT_TREE_HEIGHT);
    tree.insertLeaf(leaf);
    bytes32 root = tree.computeRoot();

    vm.prank(ROLLUP_CONTRACT);
    outbox.insert(1, root, DEFAULT_TREE_HEIGHT);

    (bytes32[] memory path,) = tree.computeSiblingPath(0);
    outbox.consume(fakeMessage, 1, 0, path);
    vm.expectRevert(abi.encodeWithSelector(Errors.Outbox__AlreadyNullified.selector, 1, 0));
    outbox.consume(fakeMessage, 1, 0, path);
  }

  function testRevertIfPathHeightMismatch() public {
    DataStructures.L2ToL1Msg memory fakeMessage = _fakeMessage(address(this));
    bytes32 leaf = fakeMessage.sha256ToField();

    NaiveMerkle tree = new NaiveMerkle(DEFAULT_TREE_HEIGHT);
    tree.insertLeaf(leaf);
    bytes32 root = tree.computeRoot();

    vm.prank(ROLLUP_CONTRACT);
    outbox.insert(1, root, DEFAULT_TREE_HEIGHT);

    NaiveMerkle biggerTree = new NaiveMerkle(DEFAULT_TREE_HEIGHT + 1);
    tree.insertLeaf(leaf);

    (bytes32[] memory path,) = biggerTree.computeSiblingPath(0);
    vm.expectRevert(
      abi.encodeWithSelector(
        Errors.Outbox__InvalidPathLength.selector, DEFAULT_TREE_HEIGHT, DEFAULT_TREE_HEIGHT + 1
      )
    );
    outbox.consume(fakeMessage, 1, 0, path);
  }

  function testRevertIfTryingToConsumeMessageNotInTree() public {
    DataStructures.L2ToL1Msg memory fakeMessage = _fakeMessage(address(this));
    bytes32 leaf = fakeMessage.sha256ToField();
    fakeMessage.content = bytes32(uint256(42069));
    bytes32 modifiedLeaf = fakeMessage.sha256ToField();

    NaiveMerkle tree = new NaiveMerkle(DEFAULT_TREE_HEIGHT);
    tree.insertLeaf(leaf);
    bytes32 root = tree.computeRoot();

    NaiveMerkle modifiedTree = new NaiveMerkle(DEFAULT_TREE_HEIGHT);
    modifiedTree.insertLeaf(modifiedLeaf);
    bytes32 modifiedRoot = modifiedTree.computeRoot();

    vm.prank(ROLLUP_CONTRACT);
    outbox.insert(1, root, DEFAULT_TREE_HEIGHT);

    (bytes32[] memory path,) = modifiedTree.computeSiblingPath(0);

    vm.expectRevert(
      abi.encodeWithSelector(
        Errors.MerkleLib__InvalidRoot.selector, root, modifiedRoot, modifiedLeaf, 0
      )
    );
    outbox.consume(fakeMessage, 1, 0, path);
  }

  function testValidInsertAndConsume() public {
    DataStructures.L2ToL1Msg memory fakeMessage = _fakeMessage(address(this));
    bytes32 leaf = fakeMessage.sha256ToField();

    NaiveMerkle tree = new NaiveMerkle(DEFAULT_TREE_HEIGHT);
    tree.insertLeaf(leaf);
    bytes32 root = tree.computeRoot();

    vm.prank(ROLLUP_CONTRACT);
    outbox.insert(1, root, DEFAULT_TREE_HEIGHT);

    (bytes32[] memory path,) = tree.computeSiblingPath(0);

    bool statusBeforeConsumption = outbox.hasMessageBeenConsumedAtBlockAndIndex(1, 0);
    assertEq(abi.encode(0), abi.encode(statusBeforeConsumption));

    vm.expectEmit(true, true, true, true, address(outbox));
    emit IOutbox.MessageConsumed(1, root, leaf, 0);
    outbox.consume(fakeMessage, 1, 0, path);

    bool statusAfterConsumption = outbox.hasMessageBeenConsumedAtBlockAndIndex(1, 0);
    assertEq(abi.encode(1), abi.encode(statusAfterConsumption));
  }

  // This test takes awhile so to keep it somewhat reasonable we've set a limit on the amount of fuzz runs
  /// forge-config: default.fuzz.runs = 64
  function testInsertAndConsumeWithVariedRecipients(
    address[256] calldata _recipients,
    uint256 _blockNumber,
    uint8 _size
  ) public {
    uint256 numberOfMessages = bound(_size, 1, _recipients.length);
    DataStructures.L2ToL1Msg[] memory messages = new DataStructures.L2ToL1Msg[](numberOfMessages);

    uint256 treeHeight = merkleTestUtil.calculateTreeHeightFromSize(numberOfMessages);
    NaiveMerkle tree = new NaiveMerkle(treeHeight);

    for (uint256 i = 0; i < numberOfMessages; i++) {
      DataStructures.L2ToL1Msg memory fakeMessage = _fakeMessage(_recipients[i]);
      messages[i] = fakeMessage;
      bytes32 modifiedLeaf = fakeMessage.sha256ToField();

      tree.insertLeaf(modifiedLeaf);
    }

    bytes32 root = tree.computeRoot();

    vm.expectEmit(true, true, true, true, address(outbox));
    emit IOutbox.RootAdded(_blockNumber, root, treeHeight);
    vm.prank(ROLLUP_CONTRACT);
    outbox.insert(_blockNumber, root, treeHeight);

    for (uint256 i = 0; i < numberOfMessages; i++) {
      (bytes32[] memory path, bytes32 leaf) = tree.computeSiblingPath(i);

      vm.expectEmit(true, true, true, true, address(outbox));
      emit IOutbox.MessageConsumed(_blockNumber, root, leaf, i);
      vm.prank(_recipients[i]);
      outbox.consume(messages[i], _blockNumber, i, path);
    }
  }

  function testCheckOutOfBoundsStatus(uint256 _blockNumber, uint256 _leafIndex) external {
    bool outOfBounds = outbox.hasMessageBeenConsumedAtBlockAndIndex(_blockNumber, _leafIndex);
    assertEq(abi.encode(0), abi.encode(outOfBounds));
  }
}

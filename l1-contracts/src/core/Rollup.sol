// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IRollup} from "./interfaces/IRollup.sol";
import {IAvailabilityOracle} from "./interfaces/IAvailabilityOracle.sol";
import {IInbox} from "./interfaces/messagebridge/IInbox.sol";
import {IOutbox} from "./interfaces/messagebridge/IOutbox.sol";
import {IRegistry} from "./interfaces/messagebridge/IRegistry.sol";
import {IVerifier} from "./interfaces/IVerifier.sol";
import {IERC20} from "@oz/token/ERC20/IERC20.sol";

// Libraries
import {HeaderLib} from "./libraries/HeaderLib.sol";
import {Hash} from "./libraries/Hash.sol";
import {Errors} from "./libraries/Errors.sol";
import {Constants} from "./libraries/ConstantsGen.sol";
import {MerkleLib} from "./libraries/MerkleLib.sol";
import {SignatureLib} from "./sequencer_selection/SignatureLib.sol";

// Contracts
import {MockVerifier} from "../mock/MockVerifier.sol";
import {Inbox} from "./messagebridge/Inbox.sol";
import {Outbox} from "./messagebridge/Outbox.sol";
import {Leonidas} from "./sequencer_selection/Leonidas.sol";

/**
 * @title Rollup
 * @author Aztec Labs
 * @notice Rollup contract that is concerned about readability and velocity of development
 * not giving a damn about gas costs.
 */
contract Rollup is Leonidas, IRollup {
  struct BlockLog {
    bytes32 archive;
    bool isProven;
  }

  IRegistry public immutable REGISTRY;
  IAvailabilityOracle public immutable AVAILABILITY_ORACLE;
  IInbox public immutable INBOX;
  IOutbox public immutable OUTBOX;
  uint256 public immutable VERSION;
  IERC20 public immutable GAS_TOKEN;

  IVerifier public verifier;

  uint256 public lastBlockTs;
  // Tracks the last time time was warped on L2 ("warp" is the testing cheatcode).
  // See https://github.com/AztecProtocol/aztec-packages/issues/1614
  uint256 public lastWarpedBlockTs;

  uint256 public pendingBlockCount;
  uint256 public provenBlockCount;

  // @todo  Validate assumption:
  //        Currently we assume that the archive root following a block is specific to the block
  //        e.g., changing any values in the block or header should in the end make its way to the archive
  //
  //        More direct approach would be storing keccak256(header) as well
  mapping(uint256 blockNumber => BlockLog log) public blocks;

  bytes32 public vkTreeRoot;

  constructor(
    IRegistry _registry,
    IAvailabilityOracle _availabilityOracle,
    IERC20 _gasToken,
    bytes32 _vkTreeRoot
  ) Leonidas(msg.sender) {
    verifier = new MockVerifier();
    REGISTRY = _registry;
    AVAILABILITY_ORACLE = _availabilityOracle;
    GAS_TOKEN = _gasToken;
    INBOX = new Inbox(address(this), Constants.L1_TO_L2_MSG_SUBTREE_HEIGHT);
    OUTBOX = new Outbox(address(this));
    vkTreeRoot = _vkTreeRoot;
    VERSION = 1;

    // Genesis block
    blocks[0] = BlockLog(bytes32(0), true);
    pendingBlockCount = 1;
    provenBlockCount = 1;
  }

  function setVerifier(address _verifier) external override(IRollup) {
    // TODO remove, only needed for testing
    verifier = IVerifier(_verifier);
  }

  function setVkTreeRoot(bytes32 _vkTreeRoot) external {
    vkTreeRoot = _vkTreeRoot;
  }

  function archive() public view returns (bytes32) {
    return blocks[pendingBlockCount - 1].archive;
  }

  function isBlockProven(uint256 _blockNumber) public view returns (bool) {
    return blocks[_blockNumber].isProven;
  }

  function archiveAt(uint256 _blockNumber) public view returns (bytes32) {
    return blocks[_blockNumber].archive;
  }

  /**
   * @notice Process an incoming L2 block and progress the state
   * @param _header - The L2 block header
   * @param _archive - A root of the archive tree after the L2 block is applied
   * @param _signatures - Signatures from the validators
   */
  function process(
    bytes calldata _header,
    bytes32 _archive,
    SignatureLib.Signature[] memory _signatures
  ) public {
    _processPendingBlock(_signatures, _archive);

    // Decode and validate header
    HeaderLib.Header memory header = HeaderLib.decode(_header);
    HeaderLib.validate(header, VERSION, lastBlockTs, archive());

    if (header.globalVariables.blockNumber != pendingBlockCount) {
      revert Errors.Rollup__InvalidBlockNumber(
        pendingBlockCount, header.globalVariables.blockNumber
      );
    }

    // Check if the data is available using availability oracle (change availability oracle if you want a different DA layer)
    if (!AVAILABILITY_ORACLE.isAvailable(header.contentCommitment.txsEffectsHash)) {
      revert Errors.Rollup__UnavailableTxs(header.contentCommitment.txsEffectsHash);
    }

    blocks[pendingBlockCount++] = BlockLog(_archive, false);

    lastBlockTs = block.timestamp;

    bytes32 inHash = INBOX.consume();
    if (header.contentCommitment.inHash != inHash) {
      revert Errors.Rollup__InvalidInHash(inHash, header.contentCommitment.inHash);
    }

    // TODO(#7218): Revert to fixed height tree for outbox, currently just providing min as interim
    // Min size = smallest path of the rollup tree + 1
    (uint256 min,) = MerkleLib.computeMinMaxPathLength(header.contentCommitment.numTxs);
    uint256 l2ToL1TreeMinHeight = min + 1;
    OUTBOX.insert(
      header.globalVariables.blockNumber, header.contentCommitment.outHash, l2ToL1TreeMinHeight
    );

    // pay the coinbase 1 gas token if it is not empty and header.totalFees is not zero
    if (header.globalVariables.coinbase != address(0) && header.totalFees > 0) {
      GAS_TOKEN.transfer(address(header.globalVariables.coinbase), header.totalFees);
    }

    emit L2BlockProcessed(header.globalVariables.blockNumber);
  }

  function process(bytes calldata _header, bytes32 _archive) external override(IRollup) {
    SignatureLib.Signature[] memory emptySignatures = new SignatureLib.Signature[](0);
    process(_header, _archive, emptySignatures);
  }

  /**
   * @notice  Submit a proof for a block in the pending chain
   *
   * @dev     Will call `_progressState` to update the proven chain. Notice this have potentially
   *          unbounded gas consumption.
   *
   * @dev     Will emit `L2ProofVerified` if the proof is valid
   *
   * @dev     Will throw if:
   *          - The block number is past the pending chain
   *          - The last archive root of the header does not match the archive root of parent block
   *          - The archive root of the header does not match the archive root of the proposed block
   *          - The proof is invalid
   *
   * @dev     We provide the `_archive` even if it could be read from storage itself because it allow for
   *          better error messages. Without passing it, we would just have a proof verification failure.
   *
   * @dev     Following the `BlockLog` struct assumption
   *
   * @param  _header - The header of the block (should match the block in the pending chain)
   * @param  _archive - The archive root of the block (should match the block in the pending chain)
   * @param  _aggregationObject - The aggregation object for the proof
   * @param  _proof - The proof to verify
   */
  function submitProof(
    bytes calldata _header,
    bytes32 _archive,
    bytes32 _proverId,
    bytes calldata _aggregationObject,
    bytes calldata _proof
  ) external override(IRollup) {
    HeaderLib.Header memory header = HeaderLib.decode(_header);

    if (header.globalVariables.blockNumber >= pendingBlockCount) {
      revert Errors.Rollup__TryingToProveNonExistingBlock();
    }

    bytes32 expectedLastArchive = blocks[header.globalVariables.blockNumber - 1].archive;
    bytes32 expectedArchive = blocks[header.globalVariables.blockNumber].archive;

    // We do it this way to provide better error messages than passing along the storage values
    // TODO(#4148) Proper genesis state. If the state is empty, we allow anything for now.
    if (expectedLastArchive != bytes32(0) && header.lastArchive.root != expectedLastArchive) {
      revert Errors.Rollup__InvalidArchive(expectedLastArchive, header.lastArchive.root);
    }

    if (_archive != expectedArchive) {
      revert Errors.Rollup__InvalidProposedArchive(expectedArchive, _archive);
    }

    bytes32[] memory publicInputs =
      new bytes32[](4 + Constants.HEADER_LENGTH + Constants.AGGREGATION_OBJECT_LENGTH);
    // the archive tree root
    publicInputs[0] = _archive;
    // this is the _next_ available leaf in the archive tree
    // normally this should be equal to the block number (since leaves are 0-indexed and blocks 1-indexed)
    // but in yarn-project/merkle-tree/src/new_tree.ts we prefill the tree so that block N is in leaf N
    publicInputs[1] = bytes32(header.globalVariables.blockNumber + 1);

    publicInputs[2] = vkTreeRoot;

    bytes32[] memory headerFields = HeaderLib.toFields(header);
    for (uint256 i = 0; i < headerFields.length; i++) {
      publicInputs[i + 3] = headerFields[i];
    }

    publicInputs[headerFields.length + 3] = _proverId;

    // the block proof is recursive, which means it comes with an aggregation object
    // this snippet copies it into the public inputs needed for verification
    // it also guards against empty _aggregationObject used with mocked proofs
    uint256 aggregationLength = _aggregationObject.length / 32;
    for (uint256 i = 0; i < Constants.AGGREGATION_OBJECT_LENGTH && i < aggregationLength; i++) {
      bytes32 part;
      assembly {
        part := calldataload(add(_aggregationObject.offset, mul(i, 32)))
      }
      publicInputs[i + 4 + Constants.HEADER_LENGTH] = part;
    }

    if (!verifier.verify(_proof, publicInputs)) {
      revert Errors.Rollup__InvalidProof();
    }

    blocks[header.globalVariables.blockNumber].isProven = true;

    _progressState();

    emit L2ProofVerified(header.globalVariables.blockNumber, _proverId);
  }

  /**
   * @notice  Progresses the state of the proven chain as far as possible
   *
   * @dev     Emits `ProgressedState` if the state is progressed
   *
   * @dev     Will continue along the pending chain as long as the blocks are proven
   *          stops at the first unproven block.
   *
   * @dev     Have a potentially unbounded gas usage. @todo Will need a bounded version, such that it cannot be
   *          used as a DOS vector.
   */
  function _progressState() internal {
    if (pendingBlockCount == provenBlockCount) {
      // We are already up to date
      return;
    }

    uint256 cachedProvenBlockCount = provenBlockCount;

    for (; cachedProvenBlockCount < pendingBlockCount; cachedProvenBlockCount++) {
      if (!blocks[cachedProvenBlockCount].isProven) {
        break;
      }
    }

    if (cachedProvenBlockCount > provenBlockCount) {
      provenBlockCount = cachedProvenBlockCount;
      emit ProgressedState(provenBlockCount, pendingBlockCount);
    }
  }
}

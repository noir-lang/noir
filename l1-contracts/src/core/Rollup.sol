// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IRollup, ITestRollup} from "./interfaces/IRollup.sol";
import {IAvailabilityOracle} from "./interfaces/IAvailabilityOracle.sol";
import {IInbox} from "./interfaces/messagebridge/IInbox.sol";
import {IOutbox} from "./interfaces/messagebridge/IOutbox.sol";
import {IRegistry} from "./interfaces/messagebridge/IRegistry.sol";
import {IVerifier} from "./interfaces/IVerifier.sol";
import {IFeeJuicePortal} from "./interfaces/IFeeJuicePortal.sol";

// Libraries
import {HeaderLib} from "./libraries/HeaderLib.sol";
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
contract Rollup is Leonidas, IRollup, ITestRollup {
  struct BlockLog {
    bytes32 archive;
    bytes32 blockHash;
    uint128 slotNumber;
    bool isProven;
  }

  // @note  The number of slots within which a block must be proven
  //        This number is currently pulled out of thin air and should be replaced when we are not blind
  // @todo  #8018
  uint256 public constant TIMELINESS_PROVING_IN_SLOTS = 100;

  IRegistry public immutable REGISTRY;
  IAvailabilityOracle public immutable AVAILABILITY_ORACLE;
  IInbox public immutable INBOX;
  IOutbox public immutable OUTBOX;
  uint256 public immutable VERSION;
  IFeeJuicePortal public immutable FEE_JUICE_PORTAL;

  IVerifier public verifier;

  uint256 public pendingBlockCount;
  uint256 public provenBlockCount;

  // @todo  Validate assumption:
  //        Currently we assume that the archive root following a block is specific to the block
  //        e.g., changing any values in the block or header should in the end make its way to the archive
  //
  //        More direct approach would be storing keccak256(header) as well
  mapping(uint256 blockNumber => BlockLog log) public blocks;

  bytes32 public vkTreeRoot;

  // @note  This should not exists, but we have it now to ensure we will not be killing the devnet with our
  //        timeliness requirements.
  bool public isDevNet = Constants.IS_DEV_NET == 1;

  // @note  Assume that all blocks up to this value are automatically proven. Speeds up bootstrapping.
  //        Testing only. This should be removed eventually.
  uint256 private assumeProvenUntilBlockNumber;

  constructor(
    IRegistry _registry,
    IAvailabilityOracle _availabilityOracle,
    IFeeJuicePortal _fpcJuicePortal,
    bytes32 _vkTreeRoot,
    address _ares
  ) Leonidas(_ares) {
    verifier = new MockVerifier();
    REGISTRY = _registry;
    AVAILABILITY_ORACLE = _availabilityOracle;
    FEE_JUICE_PORTAL = _fpcJuicePortal;
    INBOX = new Inbox(address(this), Constants.L1_TO_L2_MSG_SUBTREE_HEIGHT);
    OUTBOX = new Outbox(address(this));
    vkTreeRoot = _vkTreeRoot;
    VERSION = 1;

    // Genesis block
    blocks[0] =
      BlockLog({archive: bytes32(0), blockHash: bytes32(0), slotNumber: 0, isProven: true});
    pendingBlockCount = 1;
    provenBlockCount = 1;
  }

  /**
   * @notice  Prune the pending chain up to the last proven block
   *
   * @dev     Will revert if there is nothing to prune or if the chain is not ready to be pruned
   */
  function prune() external override(IRollup) {
    if (pendingBlockCount == provenBlockCount) {
      revert Errors.Rollup__NothingToPrune();
    }

    BlockLog storage firstPendingNotInProven = blocks[provenBlockCount];
    uint256 prunableAtSlot =
      uint256(firstPendingNotInProven.slotNumber) + TIMELINESS_PROVING_IN_SLOTS;
    uint256 currentSlot = getCurrentSlot();

    if (currentSlot < prunableAtSlot) {
      revert Errors.Rollup__NotReadyToPrune(currentSlot, prunableAtSlot);
    }

    // @note  We are not deleting the blocks, but we are "winding back" the pendingBlockCount
    //        to the last block that was proven.
    //        The reason we can do this, is that any new block proposed will overwrite a previous block
    //        so no values should "survive". It it is however slightly odd for people reading
    //        the chain separately from the contract without using pendingBlockCount as a boundary.
    pendingBlockCount = provenBlockCount;

    emit PrunedPending(provenBlockCount, pendingBlockCount);
  }

  /**
   * Sets the assumeProvenUntilBlockNumber. Only the contract deployer can set it.
   * @param blockNumber - New value.
   */
  function setAssumeProvenUntilBlockNumber(uint256 blockNumber)
    external
    override(ITestRollup)
    onlyOwner
  {
    if (blockNumber > provenBlockCount && blockNumber <= pendingBlockCount) {
      for (uint256 i = provenBlockCount; i < blockNumber; i++) {
        blocks[i].isProven = true;
        emit L2ProofVerified(i, "CHEAT");
      }
      _progressState();
    }
    assumeProvenUntilBlockNumber = blockNumber;
  }

  /**
   * @notice  Set the devnet mode
   *
   * @dev     This is only needed for testing, and should be removed
   *
   * @param _devNet - Whether or not the contract is in devnet mode
   */
  function setDevNet(bool _devNet) external override(ITestRollup) {
    isDevNet = _devNet;
  }

  /**
   * @notice  Set the verifier contract
   *
   * @dev     This is only needed for testing, and should be removed
   *
   * @param _verifier - The new verifier contract
   */
  function setVerifier(address _verifier) external override(ITestRollup) {
    verifier = IVerifier(_verifier);
  }

  /**
   * @notice  Set the vkTreeRoot
   *
   * @dev     This is only needed for testing, and should be removed
   *
   * @param _vkTreeRoot - The new vkTreeRoot to be used by proofs
   */
  function setVkTreeRoot(bytes32 _vkTreeRoot) external override(ITestRollup) {
    vkTreeRoot = _vkTreeRoot;
  }

  /**
   * @notice  Published the body and processes the block
   * @dev     This should likely be purged in the future as it is a convenience method
   * @dev     `eth_log_handlers` rely on this function
   *
   * @param _header - The L2 block header
   * @param _archive - A root of the archive tree after the L2 block is applied
   * @param _blockHash - The poseidon2 hash of the header added to the archive tree in the rollup circuit
   * @param _signatures - Signatures from the validators
   * @param _body - The body of the L2 block
   */
  function publishAndProcess(
    bytes calldata _header,
    bytes32 _archive,
    bytes32 _blockHash,
    SignatureLib.Signature[] memory _signatures,
    bytes calldata _body
  ) external override(IRollup) {
    AVAILABILITY_ORACLE.publish(_body);
    process(_header, _archive, _blockHash, _signatures);
  }

  /**
   * @notice  Published the body and processes the block
   * @dev     This should likely be purged in the future as it is a convenience method
   * @dev     `eth_log_handlers` rely on this function
   * @param _header - The L2 block header
   * @param _archive - A root of the archive tree after the L2 block is applied
   * @param _blockHash - The poseidon2 hash of the header added to the archive tree in the rollup circuit
   * @param _body - The body of the L2 block
   */
  function publishAndProcess(
    bytes calldata _header,
    bytes32 _archive,
    bytes32 _blockHash,
    bytes calldata _body
  ) external override(IRollup) {
    AVAILABILITY_ORACLE.publish(_body);
    process(_header, _archive, _blockHash);
  }

  /**
   * @notice  Submit a proof for a block in the pending chain
   *
   * @dev     TODO(#7346): Verify root proofs rather than block root when batch rollups are integrated.
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
   * @param  _proverId - The id of this block's prover
   * @param  _aggregationObject - The aggregation object for the proof
   * @param  _proof - The proof to verify
   */
  function submitBlockRootProof(
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
    // We do it this way to provide better error messages than passing along the storage values
    // TODO(#4148) Proper genesis state. If the state is empty, we allow anything for now.
    if (expectedLastArchive != bytes32(0) && header.lastArchive.root != expectedLastArchive) {
      revert Errors.Rollup__InvalidArchive(expectedLastArchive, header.lastArchive.root);
    }

    bytes32 expectedArchive = blocks[header.globalVariables.blockNumber].archive;
    if (_archive != expectedArchive) {
      revert Errors.Rollup__InvalidProposedArchive(expectedArchive, _archive);
    }

    // TODO(#7346): Currently verifying block root proofs until batch rollups fully integrated.
    // Hence the below pub inputs are BlockRootOrBlockMergePublicInputs, which are larger than
    // the planned set (RootRollupPublicInputs), for the interim.
    // Public inputs are not fully verified (TODO(#7373))

    bytes32[] memory publicInputs = new bytes32[](
      Constants.BLOCK_ROOT_OR_BLOCK_MERGE_PUBLIC_INPUTS_LENGTH + Constants.AGGREGATION_OBJECT_LENGTH
    );

    // From block_root_or_block_merge_public_inputs.nr: BlockRootOrBlockMergePublicInputs.
    // previous_archive.root: the previous archive tree root
    publicInputs[0] = expectedLastArchive;
    // previous_archive.next_available_leaf_index: the previous archive next available index
    publicInputs[1] = bytes32(header.globalVariables.blockNumber);

    // new_archive.root: the new archive tree root
    publicInputs[2] = expectedArchive;
    // this is the _next_ available leaf in the archive tree
    // normally this should be equal to the block number (since leaves are 0-indexed and blocks 1-indexed)
    // but in yarn-project/merkle-tree/src/new_tree.ts we prefill the tree so that block N is in leaf N
    // new_archive.next_available_leaf_index: the new archive next available index
    publicInputs[3] = bytes32(header.globalVariables.blockNumber + 1);

    // TODO(#7346): Currently previous block hash is unchecked, but will be checked in batch rollup (block merge -> root).
    // block-building-helpers.ts is injecting as 0 for now, replicating here.
    // previous_block_hash: the block hash just preceding this block (will eventually become the end_block_hash of the prev batch)
    publicInputs[4] = bytes32(0);

    // end_block_hash: the current block hash (will eventually become the hash of the final block proven in a batch)
    publicInputs[5] = blocks[header.globalVariables.blockNumber].blockHash;

    // For block root proof outputs, we have a block 'range' of just 1 block => start and end globals are the same
    bytes32[] memory globalVariablesFields = HeaderLib.toFields(header.globalVariables);
    for (uint256 i = 0; i < globalVariablesFields.length; i++) {
      // start_global_variables
      publicInputs[i + 6] = globalVariablesFields[i];
      // end_global_variables
      publicInputs[globalVariablesFields.length + i + 6] = globalVariablesFields[i];
    }
    // out_hash: root of this block's l2 to l1 message tree (will eventually be root of roots)
    publicInputs[24] = header.contentCommitment.outHash;

    // For block root proof outputs, we have a single recipient-value fee payment pair,
    // but the struct contains space for the max (32) => we keep 31*2=62 fields blank to represent it.
    // fees: array of recipient-value pairs, for a single block just one entry (will eventually be filled and paid out here)
    publicInputs[25] = bytes32(uint256(uint160(header.globalVariables.coinbase)));
    publicInputs[26] = bytes32(header.totalFees);
    // publicInputs[27] -> publicInputs[88] left blank for empty fee array entries

    // vk_tree_root
    publicInputs[89] = vkTreeRoot;
    // prover_id: id of current block range's prover
    publicInputs[90] = _proverId;

    // the block proof is recursive, which means it comes with an aggregation object
    // this snippet copies it into the public inputs needed for verification
    // it also guards against empty _aggregationObject used with mocked proofs
    uint256 aggregationLength = _aggregationObject.length / 32;
    for (uint256 i = 0; i < Constants.AGGREGATION_OBJECT_LENGTH && i < aggregationLength; i++) {
      bytes32 part;
      assembly {
        part := calldataload(add(_aggregationObject.offset, mul(i, 32)))
      }
      publicInputs[i + 91] = part;
    }

    if (!verifier.verify(_proof, publicInputs)) {
      revert Errors.Rollup__InvalidProof();
    }

    blocks[header.globalVariables.blockNumber].isProven = true;

    _progressState();

    emit L2ProofVerified(header.globalVariables.blockNumber, _proverId);
  }

  /**
   * @notice  Get the `isProven` flag for the block number
   *
   * @param _blockNumber - The block number to check
   *
   * @return bool - True if proven, false otherwise
   */
  function isBlockProven(uint256 _blockNumber) external view override(IRollup) returns (bool) {
    return blocks[_blockNumber].isProven;
  }

  /**
   * @notice  Get the archive root of a specific block
   *
   * @param _blockNumber - The block number to get the archive root of
   *
   * @return bytes32 - The archive root of the block
   */
  function archiveAt(uint256 _blockNumber) external view override(IRollup) returns (bytes32) {
    return blocks[_blockNumber].archive;
  }

  /**
   * @notice Processes an incoming L2 block with signatures
   *
   * @param _header - The L2 block header
   * @param _archive - A root of the archive tree after the L2 block is applied
   * @param _blockHash - The poseidon2 hash of the header added to the archive tree in the rollup circuit
   * @param _signatures - Signatures from the validators
   */
  function process(
    bytes calldata _header,
    bytes32 _archive,
    bytes32 _blockHash,
    SignatureLib.Signature[] memory _signatures
  ) public override(IRollup) {
    // Decode and validate header
    HeaderLib.Header memory header = HeaderLib.decode(_header);
    _validateHeaderForSubmissionBase(header);
    _validateHeaderForSubmissionSequencerSelection(header, _signatures, _archive);

    // As long as the header is passing validity check in `_validateHeaderForSubmissionBase` we can safely cast
    // the slot number to uint128
    blocks[pendingBlockCount++] = BlockLog({
      archive: _archive,
      blockHash: _blockHash,
      slotNumber: uint128(header.globalVariables.slotNumber),
      isProven: false
    });

    // @note  The block number here will always be >=1 as the genesis block is at 0
    bytes32 inHash = INBOX.consume(header.globalVariables.blockNumber);
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

    // @note  This should be addressed at the time of proving if sequential proving or at the time of
    //        inclusion into the proven chain otherwise. See #7622.
    if (header.globalVariables.coinbase != address(0) && header.totalFees > 0) {
      // @note  This will currently fail if there are insufficient funds in the bridge
      //        which WILL happen for the old version after an upgrade where the bridge follow.
      //        Consider allowing a failure. See #7938.
      FEE_JUICE_PORTAL.distributeFees(header.globalVariables.coinbase, header.totalFees);
    }

    emit L2BlockProcessed(header.globalVariables.blockNumber);

    // Automatically flag the block as proven if we have cheated and set assumeProvenUntilBlockNumber.
    if (header.globalVariables.blockNumber < assumeProvenUntilBlockNumber) {
      blocks[header.globalVariables.blockNumber].isProven = true;
      emit L2ProofVerified(header.globalVariables.blockNumber, "CHEAT");
      _progressState();
    }
  }

  /**
   * @notice Processes an incoming L2 block without signatures
   *
   * @param _header - The L2 block header
   * @param _archive - A root of the archive tree after the L2 block is applied
   * @param _blockHash - The poseidon2 hash of the header added to the archive tree in the rollup circuit
   */
  function process(bytes calldata _header, bytes32 _archive, bytes32 _blockHash)
    public
    override(IRollup)
  {
    SignatureLib.Signature[] memory emptySignatures = new SignatureLib.Signature[](0);
    process(_header, _archive, _blockHash, emptySignatures);
  }

  /**
   * @notice  Get the current archive root
   *
   * @return bytes32 - The current archive root
   */
  function archive() public view override(IRollup) returns (bytes32) {
    return blocks[pendingBlockCount - 1].archive;
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

  /**
   * @notice  Validate a header for submission to the pending chain (sequencer selection checks)
   *
   *          These validation checks are directly related to Leonidas.
   *          Note that while these checks are strict, they can be relaxed with some changes to
   *          message boxes.
   *
   *          Each of the following validation checks must pass, otherwise an error is thrown and we revert.
   *          - The slot MUST be the current slot
   *            This might be relaxed for allow consensus set to better handle short-term bursts of L1 congestion
   *          - The slot MUST be in the current epoch
   *
   * @dev     While in isDevNet, we allow skipping all of the checks as we simply assume only TRUSTED sequencers
   *
   * @param _header - The header to validate
   * @param _signatures - The signatures to validate
   * @param _archive - The archive root of the block
   */
  function _validateHeaderForSubmissionSequencerSelection(
    HeaderLib.Header memory _header,
    SignatureLib.Signature[] memory _signatures,
    bytes32 _archive
  ) internal {
    if (isDevNet) {
      // @note  If we are running in a devnet, we don't want to perform all the consensus
      //        checks, we instead simply require that either there are NO validators or
      //        that the proposer is a validator.
      //
      //        This means that we relaxes the condition that the block must land in the
      //        correct slot and epoch to make it more fluid for the devnet launch
      //        or for testing.
      if (getValidatorCount() == 0) {
        return;
      }

      if (!isValidator(msg.sender)) {
        revert Errors.Leonidas__InvalidProposer(address(0), msg.sender);
      }
      return;
    }

    uint256 slot = _header.globalVariables.slotNumber;

    // Ensure that the slot proposed is NOT in the future
    uint256 currentSlot = getCurrentSlot();
    if (slot != currentSlot) {
      revert Errors.HeaderLib__InvalidSlotNumber(currentSlot, slot);
    }

    // @note  We are currently enforcing that the slot is in the current epoch
    //        If this is not the case, there could potentially be a weird reorg
    //        of an entire epoch if no-one from the new epoch committee have seen
    //        those blocks or behaves as if they did not.

    uint256 epochNumber = getEpochAt(getTimestampForSlot(slot));
    uint256 currentEpoch = getCurrentEpoch();
    if (epochNumber != currentEpoch) {
      revert Errors.Rollup__InvalidEpoch(currentEpoch, epochNumber);
    }

    _processPendingBlock(epochNumber, slot, _signatures, _archive);
  }

  /**
   * @notice  Validate a header for submission to the pending chain (base checks)
   *          Base checks here being the checks that we wish to do regardless of the sequencer
   *          selection mechanism.
   *
   *         Each of the following validation checks must pass, otherwise an error is thrown and we revert.
   *          - The chain ID MUST match the current chain ID
   *          - The version MUST match the current version
   *          - The block id MUST be the next block in the chain
   *          - The last archive root in the header MUST match the current archive
   *          - The slot MUST be larger than the slot of the previous block (ensures single block per slot)
   *          - The timestamp MUST be equal to GENESIS_TIME + slot * SLOT_DURATION
   *          - The availability oracle MUST return true for availability of txsEffectsHash
   *            - This can be relaxed to happen at the time of `submitProof` instead
   *
   * @param _header - The header to validate
   */
  function _validateHeaderForSubmissionBase(HeaderLib.Header memory _header) internal view {
    if (block.chainid != _header.globalVariables.chainId) {
      revert Errors.Rollup__InvalidChainId(block.chainid, _header.globalVariables.chainId);
    }

    if (_header.globalVariables.version != VERSION) {
      revert Errors.Rollup__InvalidVersion(VERSION, _header.globalVariables.version);
    }

    if (_header.globalVariables.blockNumber != pendingBlockCount) {
      revert Errors.Rollup__InvalidBlockNumber(
        pendingBlockCount, _header.globalVariables.blockNumber
      );
    }

    // TODO(#4148) Proper genesis state. If the state is empty, we allow anything for now.
    bytes32 tipArchive = archive();
    if (tipArchive != bytes32(0) && tipArchive != _header.lastArchive.root) {
      revert Errors.Rollup__InvalidArchive(tipArchive, _header.lastArchive.root);
    }

    uint256 slot = _header.globalVariables.slotNumber;
    if (slot > type(uint128).max) {
      revert Errors.Rollup__SlotValueTooLarge(slot);
    }

    uint256 lastSlot = uint256(blocks[pendingBlockCount - 1].slotNumber);
    if (slot <= lastSlot) {
      revert Errors.Rollup__SlotAlreadyInChain(lastSlot, slot);
    }

    uint256 timestamp = getTimestampForSlot(slot);
    if (_header.globalVariables.timestamp != timestamp) {
      revert Errors.Rollup__InvalidTimestamp(timestamp, _header.globalVariables.timestamp);
    }

    // Check if the data is available using availability oracle (change availability oracle if you want a different DA layer)
    if (!AVAILABILITY_ORACLE.isAvailable(_header.contentCommitment.txsEffectsHash)) {
      revert Errors.Rollup__UnavailableTxs(_header.contentCommitment.txsEffectsHash);
    }
  }
}

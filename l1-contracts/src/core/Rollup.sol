// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

// Interfaces
import {IRollup} from "./interfaces/IRollup.sol";
import {IAvailabilityOracle} from "./interfaces/IAvailabilityOracle.sol";
import {IInbox} from "./interfaces/messagebridge/IInbox.sol";
import {IOutbox} from "./interfaces/messagebridge/IOutbox.sol";
import {IRegistry} from "./interfaces/messagebridge/IRegistry.sol";

// Libraries
import {HeaderLib} from "./libraries/HeaderLib.sol";
import {Hash} from "./libraries/Hash.sol";
import {Errors} from "./libraries/Errors.sol";
import {Constants} from "./libraries/ConstantsGen.sol";

// Contracts
import {MockVerifier} from "../mock/MockVerifier.sol";
import {Inbox} from "./messagebridge/Inbox.sol";
import {Outbox} from "./messagebridge/Outbox.sol";

/**
 * @title Rollup
 * @author Aztec Labs
 * @notice Rollup contract that is concerned about readability and velocity of development
 * not giving a damn about gas costs.
 */
contract Rollup is IRollup {
  MockVerifier public immutable VERIFIER;
  IRegistry public immutable REGISTRY;
  IAvailabilityOracle public immutable AVAILABILITY_ORACLE;
  IInbox public immutable INBOX;
  IOutbox public immutable OUTBOX;
  uint256 public immutable VERSION;

  bytes32 public archive; // Root of the archive tree
  uint256 public lastBlockTs;
  // Tracks the last time time was warped on L2 ("warp" is the testing cheatcode).
  // See https://github.com/AztecProtocol/aztec-packages/issues/1614
  uint256 public lastWarpedBlockTs;

  constructor(IRegistry _registry, IAvailabilityOracle _availabilityOracle) {
    VERIFIER = new MockVerifier();
    REGISTRY = _registry;
    AVAILABILITY_ORACLE = _availabilityOracle;
    INBOX = new Inbox(address(this), Constants.L1_TO_L2_MSG_SUBTREE_HEIGHT);
    OUTBOX = new Outbox(address(this));
    VERSION = 1;
  }

  /**
   * @notice Process an incoming L2 block and progress the state
   * @param _header - The L2 block header
   * @param _archive - A root of the archive tree after the L2 block is applied
   * @param _proof - The proof of correct execution
   */
  function process(bytes calldata _header, bytes32 _archive, bytes memory _proof)
    external
    override(IRollup)
  {
    // Decode and validate header
    HeaderLib.Header memory header = HeaderLib.decode(_header);
    HeaderLib.validate(header, VERSION, lastBlockTs, archive);

    // Check if the data is available using availability oracle (change availability oracle if you want a different DA layer)
    if (!AVAILABILITY_ORACLE.isAvailable(header.contentCommitment.txsEffectsHash)) {
      revert Errors.Rollup__UnavailableTxs(header.contentCommitment.txsEffectsHash);
    }

    bytes32[] memory publicInputs = new bytes32[](1);
    publicInputs[0] = _computePublicInputHash(_header, _archive);

    // @todo @benesjan We will need `nextAvailableLeafIndex` of archive to verify the proof. This value is equal to
    // current block number which is stored in the header (header.globalVariables.blockNumber).
    if (!VERIFIER.verify(_proof, publicInputs)) {
      revert Errors.Rollup__InvalidProof();
    }

    archive = _archive;
    lastBlockTs = block.timestamp;

    bytes32 inHash = INBOX.consume();
    if (header.contentCommitment.inHash != inHash) {
      revert Errors.Rollup__InvalidInHash(inHash, header.contentCommitment.inHash);
    }

    // We assume here that the number of L2 to L1 messages per tx is 2. Therefore we just need a tree that is one height
    // larger (as we can just extend the tree one layer down to hold all the L2 to L1 messages)
    uint256 l2ToL1TreeHeight = header.contentCommitment.txTreeHeight + 1;
    OUTBOX.insert(
      header.globalVariables.blockNumber, header.contentCommitment.outHash, l2ToL1TreeHeight
    );

    emit L2BlockProcessed(header.globalVariables.blockNumber);
  }

  function _computePublicInputHash(bytes calldata _header, bytes32 _archive)
    internal
    pure
    returns (bytes32)
  {
    return Hash.sha256ToField(bytes.concat(_header, _archive));
  }
}

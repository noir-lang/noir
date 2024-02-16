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
import {MessagesDecoder} from "./libraries/decoders/MessagesDecoder.sol";
import {Hash} from "./libraries/Hash.sol";
import {Errors} from "./libraries/Errors.sol";

// Contracts
import {MockVerifier} from "../mock/MockVerifier.sol";

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
    VERSION = 1;
  }

  /**
   * @notice Process an incoming L2 block and progress the state
   * @param _header - The L2 block header
   * @param _archive - A root of the archive tree after the L2 block is applied
   * @param _body - The L2 block body
   * @param _proof - The proof of correct execution
   */
  function process(
    bytes calldata _header,
    bytes32 _archive,
    bytes calldata _body, // TODO(#3938) Update this to pass in only th messages and not the whole body.
    bytes memory _proof
  ) external override(IRollup) {
    // Decode and validate header
    HeaderLib.Header memory header = HeaderLib.decode(_header);
    HeaderLib.validate(header, VERSION, lastBlockTs, archive);

    // Check if the data is available using availability oracle (change availability oracle if you want a different DA layer)
    if (!AVAILABILITY_ORACLE.isAvailable(header.contentCommitment.txsHash)) {
      revert Errors.Rollup__UnavailableTxs(header.contentCommitment.txsHash);
    }

    // Decode the cross-chain messages (Will be removed as part of message model change)
    (,, bytes32[] memory l1ToL2Msgs, bytes32[] memory l2ToL1Msgs) = MessagesDecoder.decode(_body);

    bytes32[] memory publicInputs = new bytes32[](1);
    publicInputs[0] = _computePublicInputHash(_header, _archive);

    // @todo @benesjan We will need `nextAvailableLeafIndex` of archive to verify the proof. This value is equal to
    // current block number which is stored in the header (header.globalVariables.blockNumber).
    if (!VERIFIER.verify(_proof, publicInputs)) {
      revert Errors.Rollup__InvalidProof();
    }

    archive = _archive;
    lastBlockTs = block.timestamp;

    // @todo (issue #605) handle fee collector
    IInbox inbox = REGISTRY.getInbox();
    inbox.batchConsume(l1ToL2Msgs, msg.sender);

    IOutbox outbox = REGISTRY.getOutbox();
    outbox.sendL1Messages(l2ToL1Msgs);

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

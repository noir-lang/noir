// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title Errors Library
 * @author Aztec Labs
 * @notice Library that contains errors used throughout the Aztec protocol
 * Errors are prefixed with the contract name to make it easy to identify where the error originated
 * when there are multiple contracts that could have thrown the error.
 */
library Errors {
  // Inbox
  error Inbox__Unauthorized(); // 0xe5336a6b
  error Inbox__ActorTooLarge(bytes32 actor); // 0xa776a06e
  error Inbox__ContentTooLarge(bytes32 content); // 0x47452014
  error Inbox__SecretHashTooLarge(bytes32 secretHash); // 0xecde7e2c
  error Inbox__MustBuildBeforeConsume(); // 0xc4901999

  // Outbox
  error Outbox__Unauthorized(); // 0x2c9490c2
  error Outbox__InvalidChainId(); // 0x577ec7c4
  error Outbox__InvalidVersion(uint256 entry, uint256 message); // 0x7915cac3
  error Outbox__NothingToConsume(bytes32 messageHash); // 0xfb4fb506
  error Outbox__IncompatibleEntryArguments(
    bytes32 messageHash,
    uint64 storedFee,
    uint64 feePassed,
    uint32 storedVersion,
    uint32 versionPassed,
    uint32 storedDeadline,
    uint32 deadlinePassed
  ); // 0x5e789f34
  error Outbox__InvalidPathLength(uint256 expected, uint256 actual); // 0x481bcd9c
  error Outbox__InsertingInvalidRoot(); // 0x73c2daca
  error Outbox__RootAlreadySetAtBlock(uint256 l2BlockNumber); // 0x3eccfd3e
  error Outbox__InvalidRecipient(address expected, address actual); // 0x57aad581
  error Outbox__AlreadyNullified(uint256 l2BlockNumber, uint256 leafIndex); // 0xfd71c2d4
  error Outbox__NothingToConsumeAtBlock(uint256 l2BlockNumber); // 0xa4508f22
  error Outbox__BlockNotProven(uint256 l2BlockNumber); // 0x0e194a6d

  // Rollup
  error Rollup__InvalidArchive(bytes32 expected, bytes32 actual); // 0xb682a40e
  error Rollup__InvalidProposedArchive(bytes32 expected, bytes32 actual); // 0x32532e73
  error Rollup__InvalidBlockNumber(uint256 expected, uint256 actual); // 0xe5edf847
  error Rollup__SlotValueTooLarge(uint256 slot); // 0x7234f4fe
  error Rollup__SlotAlreadyInChain(uint256 lastSlot, uint256 proposedSlot); // 0x83510bd0
  error Rollup__InvalidEpoch(uint256 expected, uint256 actual); // 0x3c6d65e6
  error Rollup__TryingToProveNonExistingBlock(); // 0x34ef4954
  error Rollup__InvalidInHash(bytes32 expected, bytes32 actual); // 0xcd6f4233
  error Rollup__InvalidProof(); // 0xa5b2ba17
  error Rollup__InvalidChainId(uint256 expected, uint256 actual); // 0x37b5bc12
  error Rollup__InvalidVersion(uint256 expected, uint256 actual); // 0x9ef30794
  error Rollup__InvalidTimestamp(uint256 expected, uint256 actual); // 0x3132e895
  error Rollup__TimestampInFuture(); // 0xbc1ce916
  error Rollup__TimestampTooOld(); // 0x72ed9c81
  error Rollup__UnavailableTxs(bytes32 txsHash); // 0x414906c3
  error Rollup__NothingToPrune(); // 0x850defd3
  error Rollup__NotReadyToPrune(uint256 currentSlot, uint256 prunableAt); // 0x9fdf1614

  // Registry
  error Registry__RollupNotRegistered(address rollup); // 0xa1fee4cf
  error Registry__RollupAlreadyRegistered(address rollup); // 0x3c34eabf

  //TxsDecoder
  error TxsDecoder__InvalidLogsLength(uint256 expected, uint256 actual); // 0x829ca981
  error TxsDecoder__TxsTooLarge(uint256 expected, uint256 actual); // 0xc7d44a62

  // HeaderLib
  error HeaderLib__InvalidHeaderSize(uint256 expected, uint256 actual); // 0xf3ccb247
  error HeaderLib__InvalidSlotNumber(uint256 expected, uint256 actual); // 0x09ba91ff

  // MerkleLib
  error MerkleLib__InvalidRoot(bytes32 expected, bytes32 actual, bytes32 leaf, uint256 leafIndex); // 0x5f216bf1

  // SignatureLib
  error SignatureLib__CannotVerifyEmpty(); // 0xc7690a37
  error SignatureLib__InvalidSignature(address expected, address recovered); // 0xd9cbae6c

  // SampleLib
  error SampleLib__IndexOutOfBounds(uint256 requested, uint256 bound); // 0xa12fc559

  // Sequencer Selection (Leonidas)
  error Leonidas__EpochNotSetup(); // 0xcf4e597e
  error Leonidas__InvalidProposer(address expected, address actual); // 0xd02d278e
  error Leonidas__InsufficientAttestations(uint256 minimumNeeded, uint256 provided); // 0xbf1ca4cb
  error Leonidas__InsufficientAttestationsProvided(uint256 minimumNeeded, uint256 provided); // 0x2e7debe9

  // Fee Juice Portal
  error FeeJuicePortal__AlreadyInitialized(); // 0xc7a172fe
  error FeeJuicePortal__InvalidInitialization(); // 0xfd9b3208
  error FeeJuicePortal__Unauthorized(); // 0x67e3691e
}

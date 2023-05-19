// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

library Errors {
  // Inbox
  error Inbox__DeadlineBeforeNow();
  error Inbox__NotPastDeadline();
  error Inbox__PastDeadline();
  error Inbox__FeeTooHigh();
  error Inbox__FailedToWithdrawFees();
  error Inbox__Unauthorized();
  error Inbox__NothingToConsume(bytes32 entryKey);
  error Inbox__IncompatibleEntryArguments(
    bytes32 entryKey,
    uint64 storedFee,
    uint64 feePassed,
    uint32 storedDeadline,
    uint32 deadlinePassed
  );

  // Outbox
  error Outbox__Unauthorized();
  error Outbox__InvalidChainId();
  error Outbox__NothingToConsume(bytes32 entryKey);
  error Outbox__IncompatibleEntryArguments(
    bytes32 entryKey,
    uint64 storedFee,
    uint64 feePassed,
    uint32 storedDeadline,
    uint32 deadlinePassed
  );

  // Rollup
  error Rollup__InvalidStateHash(bytes32 expected, bytes32 actual);
  error Rollup__InvalidProof();
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

library DataStructures {
  struct OutboxMessageMetadata {
    uint256 _l2BlockNumber;
    uint256 _leafIndex;
    bytes32[] _path;
  }
}

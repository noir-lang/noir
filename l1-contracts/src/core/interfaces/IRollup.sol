// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

interface IRollup {
  function process(bytes memory _proof, bytes calldata _l2Block) external;
}

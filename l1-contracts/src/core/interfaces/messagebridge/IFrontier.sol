// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

interface IFrontier {
  /**
   * @notice Inserts a new leaf into the frontier tree and returns its index
   * @param _leaf - The leaf to insert
   * @return The index of the leaf in the tree
   */
  function insertLeaf(bytes32 _leaf) external returns (uint256);

  /**
   * @notice Returns the root of the frontier tree
   * @return The root of the tree
   */
  function root() external view returns (bytes32);

  /**
   * @notice Returns whether the tree is full
   * @return True if full, false otherwise
   */
  function isFull() external view returns (bool);
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

interface ILeonidas {
  // Changing depending on sybil mechanism and slashing enforcement
  function addValidator(address _validator) external;
  function removeValidator(address _validator) external;

  // Likely changing to optimize in Pleistarchus
  function setupEpoch() external;
  function getCurrentProposer() external view returns (address);

  // Stable
  function getCurrentEpoch() external view returns (uint256);
  function getCurrentSlot() external view returns (uint256);

  // Consider removing below this point
  // Likely removal of these to replace with a size and indiviual getter
  function getEpochCommittee(uint256 _epoch) external view returns (address[] memory);
  function getValidators() external view returns (address[] memory);
}

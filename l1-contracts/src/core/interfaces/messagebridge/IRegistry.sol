// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.18;

import {DataStructures} from "../../libraries/DataStructures.sol";
import {IRollup} from "../IRollup.sol";

interface IRegistry {
  // docs:start:registry_upgrade
  function upgrade(address _rollup) external returns (uint256);
  // docs:end:registry_upgrade

  // docs:start:registry_get_rollup
  function getRollup() external view returns (IRollup);
  // docs:end:registry_get_rollup

  // docs:start:registry_get_version_for
  function getVersionFor(address _rollup) external view returns (uint256);
  // docs:end:registry_get_version_for

  // docs:start:registry_get_snapshot
  function getSnapshot(uint256 _version)
    external
    view
    returns (DataStructures.RegistrySnapshot memory);
  // docs:end:registry_get_snapshot

  // docs:start:registry_get_current_snapshot
  function getCurrentSnapshot() external view returns (DataStructures.RegistrySnapshot memory);
  // docs:end:registry_get_current_snapshot

  // docs:start:registry_number_of_versions
  function numberOfVersions() external view returns (uint256);
  // docs:end:registry_number_of_versions

  function isRollupRegistered(address _rollup) external view returns (bool);
}

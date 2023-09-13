// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.18;

import {DataStructures} from "../../libraries/DataStructures.sol";
import {IRollup} from "../IRollup.sol";
import {IInbox} from "./IInbox.sol";
import {IOutbox} from "./IOutbox.sol";

interface IRegistry {
  // docs:start:registry_upgrade
  function upgrade(address _rollup, address _inbox, address _outbox) external returns (uint256);
  // docs:end:registry_upgrade

  // docs:start:registry_get_rollup
  function getRollup() external view returns (IRollup);
  // docs:end:registry_get_rollup

  // docs:start:registry_get_version_for
  function getVersionFor(address _rollup) external view returns (uint256);
  // docs:end:registry_get_version_for

  // docs:start:registry_get_inbox
  function getInbox() external view returns (IInbox);
  // docs:end:registry_get_inbox

  // docs:start:registry_get_outbox
  function getOutbox() external view returns (IOutbox);
  // docs:end:registry_get_outbox

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
}

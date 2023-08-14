// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.18;

import {DataStructures} from "../../libraries/DataStructures.sol";
import {IRollup} from "../IRollup.sol";
import {IInbox} from "./IInbox.sol";
import {IOutbox} from "./IOutbox.sol";

interface IRegistry {
  function upgrade(address _rollup, address _inbox, address _outbox) external returns (uint256);

  function getRollup() external view returns (IRollup);

  function getVersionFor(address _rollup) external view returns (uint256);

  function getInbox() external view returns (IInbox);

  function getOutbox() external view returns (IOutbox);

  function getSnapshot(uint256 _version)
    external
    view
    returns (DataStructures.RegistrySnapshot memory);

  function getCurrentSnapshot() external view returns (DataStructures.RegistrySnapshot memory);

  function numberOfVersions() external view returns (uint256);
}

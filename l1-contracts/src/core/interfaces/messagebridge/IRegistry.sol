// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.18;

import {DataStructures} from "../../libraries/DataStructures.sol";
import {IRollup} from "../IRollup.sol";
import {IInbox} from "./IInbox.sol";
import {IOutbox} from "./IOutbox.sol";

interface IRegistry {
  function getL1L2Addresses() external view returns (DataStructures.L1L2Addresses memory);

  function getRollup() external view returns (IRollup);

  function getInbox() external view returns (IInbox);

  function getOutbox() external view returns (IOutbox);
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {IRegistry} from "@aztec/core/interfaces/messagebridge/IRegistry.sol";
import {IRollup} from "@aztec/core/interfaces/IRollup.sol";
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {IOutbox} from "@aztec/core/interfaces/messagebridge/IOutbox.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";

/**
 * @title Registry
 * @author Aztec Labs
 * @notice Keeps track of important information for L1<>L2 communication.
 */
contract Registry is IRegistry {
  // TODO(rahul) - https://github.com/AztecProtocol/aztec-packages/issues/526
  // Need to create a snashot of addresses per version!

  DataStructures.L1L2Addresses public addresses;

  function setAddresses(address _rollup, address _inbox, address _outbox) public {
    addresses = DataStructures.L1L2Addresses(_rollup, _inbox, _outbox);
  }

  function getL1L2Addresses() external view override returns (DataStructures.L1L2Addresses memory) {
    return addresses;
  }

  function getRollup() external view override returns (IRollup) {
    return IRollup(addresses.rollup);
  }

  function getInbox() external view override returns (IInbox) {
    return IInbox(addresses.inbox);
  }

  function getOutbox() external view override returns (IOutbox) {
    return IOutbox(addresses.outbox);
  }
}

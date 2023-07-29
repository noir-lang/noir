// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";
import {IInbox} from "@aztec/core/interfaces/messagebridge/IInbox.sol";
import {Inbox} from "@aztec/core/messagebridge/Inbox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";
import {Errors} from "@aztec/core/libraries/Errors.sol";

import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {MessageBox} from "@aztec/core/libraries/MessageBox.sol";

contract RegistryTest is Test {
  address internal constant DEAD = address(0xdead);

  Registry internal registry;

  function setUp() public {
    registry = new Registry();
  }

  function testConstructorSetup() public {
    assertEq(registry.numberOfVersions(), 1, "should have 1 version");
    DataStructures.RegistrySnapshot memory snapshot = registry.getCurrentSnapshot();
    assertEq(snapshot.rollup, DEAD, "should have dead rollup");
    assertEq(snapshot.inbox, DEAD, "should have dead inbox");
    assertEq(snapshot.outbox, DEAD, "should have dead outbox");
    assertEq(address(registry.getRollup()), DEAD);
    assertEq(address(registry.getInbox()), DEAD);
    assertEq(address(registry.getOutbox()), DEAD);

    vm.expectRevert(abi.encodeWithSelector(Errors.Registry__RollupNotRegistered.selector, DEAD));
    registry.getVersionFor(DEAD);
  }

  function testUpgrade() public {
    address newRollup = address(0xbeef1);
    address newInbox = address(0xbeef2);
    address newOutbox = address(0xbeef3);
    uint256 newVersion = registry.upgrade(newRollup, newInbox, newOutbox);

    assertEq(registry.numberOfVersions(), 2, "should have 2 versions");
    DataStructures.RegistrySnapshot memory snapshot = registry.getCurrentSnapshot();
    assertEq(snapshot.rollup, newRollup, "should have newRollup");
    assertEq(snapshot.inbox, newInbox, "should have newInbox");
    assertEq(snapshot.outbox, newOutbox, "should have newOutbox");

    assertEq(address(registry.getRollup()), newRollup);
    assertEq(address(registry.getInbox()), newInbox);
    assertEq(address(registry.getOutbox()), newOutbox);
    assertEq(
      registry.getVersionFor(newRollup), newVersion, "should have version newVersion for newRollup"
    );
  }

  function testRevertUpgradeToSame() public {
    registry.upgrade(DEAD, DEAD, DEAD);
    vm.expectRevert(abi.encodeWithSelector(Errors.Registry__RollupAlreadyRegistered.selector, DEAD));
    registry.upgrade(DEAD, DEAD, DEAD);
  }

  function testRevertGetVersionForNonExistent() public {
    address rollup = address(0xbeef1);
    vm.expectRevert(abi.encodeWithSelector(Errors.Registry__RollupNotRegistered.selector, rollup));
    registry.getVersionFor(rollup);
  }
}

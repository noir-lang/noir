// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";
import {Ownable} from "@oz/access/Ownable.sol";
import {Registry} from "../src/core/messagebridge/Registry.sol";
import {Errors} from "../src/core/libraries/Errors.sol";

import {DataStructures} from "../src/core/libraries/DataStructures.sol";

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
    assertEq(address(registry.getRollup()), DEAD);

    vm.expectRevert(abi.encodeWithSelector(Errors.Registry__RollupNotRegistered.selector, DEAD));
    registry.getVersionFor(DEAD);
  }

  function testUpgrade() public {
    address newRollup = address(0xbeef1);
    uint256 newVersion = registry.upgrade(newRollup);

    assertEq(registry.numberOfVersions(), 2, "should have 2 versions");
    DataStructures.RegistrySnapshot memory snapshot = registry.getCurrentSnapshot();
    assertEq(snapshot.rollup, newRollup, "should have newRollup");

    assertEq(address(registry.getRollup()), newRollup);
    assertEq(
      registry.getVersionFor(newRollup), newVersion, "should have version newVersion for newRollup"
    );
  }

  function testRevertUpgradeToSame() public {
    registry.upgrade(DEAD);
    vm.expectRevert(abi.encodeWithSelector(Errors.Registry__RollupAlreadyRegistered.selector, DEAD));
    registry.upgrade(DEAD);
  }

  function testRevertGetVersionForNonExistent() public {
    address rollup = address(0xbeef1);
    vm.expectRevert(abi.encodeWithSelector(Errors.Registry__RollupNotRegistered.selector, rollup));
    registry.getVersionFor(rollup);
  }

  function testRevertUpgradeAsNonOwner(address _owner) public {
    vm.assume(_owner != registry.owner());
    vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, _owner));
    vm.prank(_owner);
    registry.upgrade(DEAD);
  }
}

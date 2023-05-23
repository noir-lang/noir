// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

import {Rollup} from "@aztec/core/Rollup.sol";
import {Outbox} from "@aztec/core/messagebridge/Outbox.sol";
import {Inbox} from "@aztec/core/messagebridge/Inbox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";
import {Errors} from "@aztec/core/libraries/Errors.sol";
import {RollupNativeAsset} from "./RollupNativeAsset.sol";

contract RNATest is Test {
  event MessageConsumed(bytes32 indexed entryKey, address indexed recipient);

  Rollup internal rollup;
  RollupNativeAsset internal rna;

  Registry internal registry;
  Outbox internal outbox;
  Inbox internal inbox;

  // Using a run of e2e_rollup_native_asset_contract.test.ts to get values
  // for entryKeys and aztecAddress
  bytes32 internal constant AZTEC_ADDRESS =
    0x06b5fb872b0b08560791085e14039c2c23c3ba6591cc21ae067cd9036461d032;
  bytes32 internal constant ENTRY_KEY =
    0x1aa9c93cf0144c36f3242e9983e11eb153bbc401340c8777e49e28093ac88b86;

  function setUp() public {
    registry = new Registry();
    outbox = new Outbox(address(registry));
    inbox = new Inbox(address(registry));
    rollup = new Rollup(registry);

    registry.upgrade(address(rollup), address(inbox), address(outbox));

    rna = new RollupNativeAsset();
    // Essentially deploying the rna contract on the 0xbeef address to make matching entry easy.
    vm.etch(address(0xbeef), address(rna).code);
    rna = RollupNativeAsset(address(0xbeef));

    rna.initialize(address(registry), AZTEC_ADDRESS);
  }

  function testWithdraw() public {
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = ENTRY_KEY;

    // Insert messages into the outbox (impersonating the rollup contract)
    vm.prank(address(rollup));
    outbox.sendL1Messages(entryKeys);

    assertEq(rna.balanceOf(address(0xdead)), 0);

    vm.expectEmit(true, true, true, true);
    emit MessageConsumed(ENTRY_KEY, address(rna));
    bytes32 entryKey = rna.withdraw(654, address(0xdead));
    // Should have received 654 RNA tokens
    assertEq(rna.balanceOf(address(0xdead)), 654);

    // Should not be able to withdraw again
    vm.expectRevert(abi.encodeWithSelector(Errors.Outbox__NothingToConsume.selector, entryKey));
    rna.withdraw(654, address(0xdead));
  }
}

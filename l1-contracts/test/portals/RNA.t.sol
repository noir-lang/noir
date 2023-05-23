// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

import {Rollup} from "@aztec/core/Rollup.sol";
import {Outbox} from "@aztec/core/messagebridge/Outbox.sol";
import {Inbox} from "@aztec/core/messagebridge/Inbox.sol";
import {Registry} from "@aztec/core/messagebridge/Registry.sol";
import {Errors} from "@aztec/core/libraries/Errors.sol";
import {DataStructures} from "@aztec/core/libraries/DataStructures.sol";
import {Hash} from "@aztec/core/libraries/Hash.sol";
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
    0x1647b194c649f5dd01d7c832f89b0f496043c9150797923ea89e93d5ac619a93;

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
    uint256 withdrawAmount = 654;
    address _recipient = address(0xdead);
    bytes32[] memory entryKeys = new bytes32[](1);
    entryKeys[0] = outbox.computeEntryKey(
      DataStructures.L2ToL1Msg({
        sender: DataStructures.L2Actor({actor: AZTEC_ADDRESS, version: 1}),
        recipient: DataStructures.L1Actor({actor: address(rna), chainId: block.chainid}),
        content: Hash.sha256ToField(
          abi.encodeWithSignature("withdraw(uint256,address)", withdrawAmount, _recipient)
          )
      })
    );

    // Insert messages into the outbox (impersonating the rollup contract)
    vm.prank(address(rollup));
    outbox.sendL1Messages(entryKeys);

    assertEq(rna.balanceOf(_recipient), 0);

    vm.expectEmit(true, true, true, true);
    emit MessageConsumed(entryKeys[0], address(rna));
    bytes32 entryKey = rna.withdraw(withdrawAmount, _recipient);
    // Should have received 654 RNA tokens
    assertEq(rna.balanceOf(_recipient), withdrawAmount);

    // Should not be able to withdraw again
    vm.expectRevert(abi.encodeWithSelector(Errors.Outbox__NothingToConsume.selector, entryKey));
    rna.withdraw(withdrawAmount, _recipient);
  }
}

// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

import {FrontierMerkle} from "./../src/core/messagebridge/frontier_tree/Frontier.sol";
import {Constants} from "../src/core/libraries/ConstantsGen.sol";

contract ParityTest is Test {
  function setUp() public {}

  // Checks whether sha root matches output of base parity circuit
  function testRootMatchesBaseParity() public {
    // matches noir-protocol-circuits/crates/parity-lib/src/base/base_parity_inputs.nr
    uint248[4] memory msgs = [
      0x151de48ca3efbae39f180fe00b8f472ec9f25be10b4f283a87c6d783935370,
      0x14c2ea9dedf77698d4afe23bc663263eed0bf9aa3a8b17d9b74812f185610f,
      0x1570cc6641699e3ae87fa258d80a6d853f7b8ccb211dc244d017e2ca6530f8,
      0x2806c860af67e9cd50000378411b8c4c4db172ceb2daa862b259b689ccbdc1
    ];

    // We can't use Constants.NUM_MSGS_PER_BASE_PARITY directly when defining the array so we do the check here to
    // ensure it does not get outdated.
    assertEq(
      msgs.length,
      Constants.NUM_MSGS_PER_BASE_PARITY,
      "NUM_MSGS_PER_BASE_PARITY changed, update msgs."
    );

    uint256 treeHeight = 2; // log_2(NUM_MSGS_PER_BASE_PARITY)
    // We don't have log_2 directly accessible in solidity so I just do the following check here to ensure
    // the hardcoded value is not outdated.
    assertEq(
      2 ** treeHeight,
      Constants.NUM_MSGS_PER_BASE_PARITY,
      "Base parity circuit subtree height changed, update treeHeight."
    );

    FrontierMerkle frontier = new FrontierMerkle(treeHeight);

    for (uint256 i = 0; i < msgs.length; i++) {
      frontier.insertLeaf(bytes32(bytes.concat(new bytes(1), bytes31(msgs[i]))));
    }

    bytes32 expectedRoot = 0x00fc986d54a5e0af4f6e0d49399b9806c2b225e6c652fa5a831ecf6c6c29719d;
    assertEq(frontier.root(), expectedRoot, "Root does not match base parity circuit root");
  }

  // Checks whether sha root matches output of root parity circuit
  function testRootMatchesRootParity() public {
    // sha256 roots coming out of base parity circuits
    // matches noir-protocol-circuits/crates/parity-lib/src/root/root_parity_inputs.nr
    uint248[4] memory baseRoots = [
      0xb3a3fc1968999f2c2d798b900bdf0de41311be2a4d20496a7e792a521fc8ab,
      0x43f78e0ebc9633ce336a8c086064d898c32fb5d7d6011f5427459c0b8d14e9,
      0x024259b6404280addcc9319bc5a32c9a5d56af5c93b2f941fa326064fbe963,
      0x53042d820859d80c474d4694e03778f8dc0ac88fc1c3a97b4369c1096e904a
    ];

    // We can't use Constants.NUM_BASE_PARITY_PER_ROOT_PARITY directly when defining the array so we do the check here
    // to ensure it does not get outdated.
    assertEq(
      baseRoots.length,
      Constants.NUM_BASE_PARITY_PER_ROOT_PARITY,
      "NUM_BASE_PARITY_PER_ROOT_PARITY changed, update baseRoots."
    );

    uint256 treeHeight = 2; // log_2(NUM_BASE_PARITY_PER_ROOT_PARITY)
    // We don't have log_2 directly accessible in solidity so I just do the following check here to ensure
    // the hardcoded value is not outdated.
    assertEq(
      2 ** treeHeight,
      Constants.NUM_BASE_PARITY_PER_ROOT_PARITY,
      "Root parity circuit subtree height changed, update treeHeight."
    );

    FrontierMerkle frontier = new FrontierMerkle(treeHeight);

    for (uint256 i = 0; i < baseRoots.length; i++) {
      frontier.insertLeaf(bytes32(bytes.concat(new bytes(1), bytes31(baseRoots[i]))));
    }

    bytes32 expectedRoot = 0x00a0c56543aa73140e5ca27231eee3107bd4e11d62164feb411d77c9d9b2da47;
    assertEq(frontier.root(), expectedRoot, "Root does not match root parity circuit root");
  }
}

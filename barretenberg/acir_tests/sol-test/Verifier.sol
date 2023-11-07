// THIS FILE WILL NOT COMPILE BY ITSELF
// Compilation is handled in `src/index.js` where solcjs gathers the dependencies

// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

import {UltraVerificationKey} from "./Key.sol";
import {BaseUltraVerifier} from "./BaseUltraVerifier.sol";

contract Verifier is BaseUltraVerifier {
    function getVerificationKeyHash() public pure override(BaseUltraVerifier) returns (bytes32) {
        return UltraVerificationKey.verificationKeyHash();
    }

    function loadVerificationKey(uint256 vk, uint256 _omegaInverseLoc) internal pure virtual override(BaseUltraVerifier) {
        UltraVerificationKey.loadVerificationKey(vk, _omegaInverseLoc);
    }
}
// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec
pragma solidity >=0.8.4;

import {Add2UltraVerificationKey as VK} from "../keys/Add2UltraVerificationKey.sol";
import {BaseUltraVerifier as BASE} from "../BaseUltraVerifier.sol";

contract Add2UltraVerifier is BASE {
    function getVerificationKeyHash() public pure override(BASE) returns (bytes32) {
        return VK.verificationKeyHash();
    }

    function loadVerificationKey(uint256 vk, uint256 _omegaInverseLoc) internal pure virtual override(BASE) {
        VK.loadVerificationKey(vk, _omegaInverseLoc);
    }
}

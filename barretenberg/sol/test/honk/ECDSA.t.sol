// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

import {TestBaseHonk} from "./TestBaseHonk.sol";
import {EcdsaHonkVerifier} from "../../src/honk/instance/EcdsaHonk.sol";
import {DifferentialFuzzer} from "../base/DifferentialFuzzer.sol";
import {IVerifier} from "../../src/interfaces/IVerifier.sol";

contract EcdsaHonkTest is TestBaseHonk {
    function setUp() public override(TestBaseHonk) {
        super.setUp();

        verifier = IVerifier(address(new EcdsaHonkVerifier()));
        fuzzer = fuzzer.with_circuit_flavour(DifferentialFuzzer.CircuitFlavour.Ecdsa);

        PUBLIC_INPUT_COUNT = 6;

        // Add default inputs to the fuzzer (we will override these in fuzz test)
        uint256[] memory inputs = new uint256[](6);
        inputs[0] = uint256(0x67);
        inputs[1] = uint256(0x6f);
        inputs[2] = uint256(0x62);
        inputs[3] = uint256(0x6c);
        inputs[4] = uint256(0x69);
        inputs[5] = uint256(0x6e);

        fuzzer = fuzzer.with_inputs(inputs);
    }

    function testFuzzProof() public {
        // NOTE we do not fuzz here yet
        // "goblin"
        // 67 6f 62 6c 69 6e
        uint256[] memory inputs = new uint256[](6);
        inputs[0] = uint256(0x67);
        inputs[1] = uint256(0x6f);
        inputs[2] = uint256(0x62);
        inputs[3] = uint256(0x6c);
        inputs[4] = uint256(0x69);
        inputs[5] = uint256(0x6e);

        // Construct Ecdsa siganture
        bytes memory proofData = fuzzer.with_inputs(inputs).generate_proof();
        (bytes32[] memory publicInputs, bytes memory proof) = splitProofHonk(proofData, PUBLIC_INPUT_COUNT);

        assertTrue(verifier.verify(proof, publicInputs), "The proof is not valid");
    }
}

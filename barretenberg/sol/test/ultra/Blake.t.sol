// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

import {TestBaseUltra} from "./TestBaseUltra.sol";
import {BlakeUltraVerifier} from "../../src/ultra/instance/BlakeUltraVerifier.sol";
import {DifferentialFuzzer} from "../base/DifferentialFuzzer.sol";
import {IVerifier} from "../../src/interfaces/IVerifier.sol";

contract BlakeUltraTest is TestBaseUltra {
    function setUp() public override(TestBaseUltra) {
        super.setUp();

        verifier = IVerifier(address(new BlakeUltraVerifier()));
        fuzzer = fuzzer.with_circuit_flavour(DifferentialFuzzer.CircuitFlavour.Blake);

        PUBLIC_INPUT_COUNT = 4;

        // Add default inputs to the fuzzer (we will override these in fuzz test)
        uint256[] memory defaultInputs = new uint256[](4);
        defaultInputs[0] = 1;
        defaultInputs[1] = 2;
        defaultInputs[2] = 3;
        defaultInputs[3] = 4;

        fuzzer = fuzzer.with_inputs(defaultInputs);
    }

    function testFuzzProof(uint256 input1, uint256 input2, uint256 input3, uint256 input4) public {
        uint256[] memory inputs = new uint256[](4);
        inputs[0] = input1;
        inputs[1] = input2;
        inputs[2] = input3;
        inputs[3] = input4;

        bytes memory proofData = fuzzer.with_inputs(inputs).generate_proof();

        (bytes32[] memory publicInputs, bytes memory proof) = splitProof(proofData, PUBLIC_INPUT_COUNT);

        assertTrue(verifier.verify(proof, publicInputs), "The proof is not valid");
    }
}

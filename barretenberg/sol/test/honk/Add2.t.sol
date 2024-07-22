// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.21;

import {TestBaseHonk} from "./TestBaseHonk.sol";

// TODO(md): need to generalize the verifier instances
import {Add2HonkVerifier} from "../../src/honk/instance/Add2Honk.sol";
import {DifferentialFuzzer} from "../base/DifferentialFuzzer.sol";
import {IVerifier} from "../../src/interfaces/IVerifier.sol";

import "forge-std/console.sol";

contract Add2HonkTest is TestBaseHonk {
    function setUp() public override(TestBaseHonk) {
        super.setUp();

        verifier = IVerifier(address(new Add2HonkVerifier()));
        fuzzer = fuzzer.with_circuit_flavour(DifferentialFuzzer.CircuitFlavour.Add2);

        PUBLIC_INPUT_COUNT = 3;

        // Add default inputs to the fuzzer (we will override these in fuzz test)
        uint256[] memory defaultInputs = new uint256[](3);
        defaultInputs[0] = 5;
        defaultInputs[1] = 10;
        defaultInputs[2] = 15;

        fuzzer = fuzzer.with_inputs(defaultInputs);
    }

    function testFuzzProof(uint16 input1, uint16 input2) public {
        uint256[] memory inputs = new uint256[](3);
        inputs[0] = uint256(input1);
        inputs[1] = uint256(input2);
        inputs[2] = inputs[0] + inputs[1];

        bytes memory proofData = fuzzer.with_inputs(inputs).generate_proof();

        (bytes32[] memory publicInputs, bytes memory proof) = splitProofHonk(proofData, PUBLIC_INPUT_COUNT);

        assertTrue(verifier.verify(proof, publicInputs), "The proof is not valid");

        console.log("Proof verified");
    }
}

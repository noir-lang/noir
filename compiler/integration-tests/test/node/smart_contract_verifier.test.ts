import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve } from 'path';
import toml from 'toml';

import { compile, init_log_level as compilerLogLevel } from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { separatePublicInputsFromProof } from '../shared/proof';

compilerLogLevel('INFO');

const test_cases = [
  {
    case: 'tooling/nargo_cli/tests/execution_success/1_mul',
    compiled: 'contracts/1_mul.sol:UltraVerifier',
    numPublicInputs: 0,
  },
  {
    case: 'compiler/integration-tests/circuits/main',
    compiled: 'contracts/main.sol:UltraVerifier',
    numPublicInputs: 1,
  },
];

test_cases.forEach((testInfo) => {
  const test_name = testInfo.case.split('/').pop();

  it(`${test_name} (smart contract verifier)`, async () => {
    const base_relative_path = '../..';
    const test_case = testInfo.case;

    const noir_source_path = resolve(`${base_relative_path}/${test_case}/src/main.nr`);

    const compile_output = compile(noir_source_path);

    const noir_program = { bytecode: compile_output.circuit, abi: compile_output.abi };
    const backend = new BarretenbergBackend(noir_program);
    const program = new Noir(noir_program, backend);

    // JS Proving

    const prover_toml = readFileSync(resolve(`${base_relative_path}/${test_case}/Prover.toml`)).toString();
    const inputs = toml.parse(prover_toml);

    const proofWithPublicInputs = await program.generateFinalProof(inputs);

    // JS verification

    const verified = await program.verifyFinalProof(proofWithPublicInputs);
    expect(verified, 'Proof fails verification in JS').to.be.true;

    // Smart contract verification

    const contract = await ethers.deployContract(testInfo.compiled, [], {});

    const { proof, publicInputs } = separatePublicInputsFromProof(proofWithPublicInputs, testInfo.numPublicInputs);
    const result = await contract.verify(proof, publicInputs);

    expect(result).to.be.true;
  });
});

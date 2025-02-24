import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve } from 'path';
import toml from 'toml';

import { Noir } from '@noir-lang/noir_js';
import { UltraPlonkBackend } from '@aztec/bb.js';

import { compile, createFileManager } from '@noir-lang/noir_wasm';

const test_cases = [
  {
    case: 'test_programs/execution_success/1_mul',
    compiled: 'contracts/1_mul.sol:UltraVerifier',
    numPublicInputs: 0,
  },
  {
    case: 'test_programs/execution_success/assert_statement',
    compiled: 'contracts/assert_statement.sol:UltraVerifier',
    numPublicInputs: 1,
  },
];

test_cases.forEach((testInfo) => {
  const test_name = testInfo.case.split('/').pop();

  it(`${test_name} (smart contract verifier)`, async () => {
    const base_relative_path = '../..';
    const test_case = testInfo.case;

    const fm = createFileManager(resolve(`${base_relative_path}/${test_case}`));
    const compileResult = await compile(fm);
    if (!('program' in compileResult)) {
      throw new Error('Compilation failed');
    }

    const noir_program = compileResult.program;

    const program = new Noir(noir_program);

    // JS Proving

    const prover_toml = readFileSync(resolve(`${base_relative_path}/${test_case}/Prover.toml`)).toString();
    const inputs = toml.parse(prover_toml);
    const { witness } = await program.execute(inputs);

    const backend = new UltraPlonkBackend(noir_program.bytecode, {}, { recursive: false });
    const proofData = await backend.generateProof(witness);

    // JS verification

    const verified = await backend.verifyProof(proofData);
    expect(verified, 'Proof fails verification in JS').to.be.true;

    // Smart contract verification

    const contract = await ethers.deployContract(testInfo.compiled, []);

    const result = await contract.verify(proofData.proof, proofData.publicInputs);

    expect(result).to.be.true;
  });
});

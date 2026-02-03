import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve } from 'path';
import toml from 'toml';

import { Noir } from '@noir-lang/noir_js';
import { Barretenberg, UltraHonkBackend } from '@aztec/bb.js';

import { compile, createFileManager } from '@noir-lang/noir_wasm';

const test_cases = [
  {
    case: 'test_programs/execution_success/a_1_mul',
    compiled: 'contracts/a_1_mul.sol:HonkVerifier',
    zk_lib: 'contracts/a_1_mul.sol:ZKTranscriptLib',
    numPublicInputs: 0,
  },
  {
    case: 'test_programs/execution_success/assert_statement',
    compiled: 'contracts/assert_statement.sol:HonkVerifier',
    zk_lib: 'contracts/assert_statement.sol:ZKTranscriptLib',
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
    const barretenbergAPI = await Barretenberg.new();
    const backend = new UltraHonkBackend(noir_program.bytecode, barretenbergAPI);
    const proofData = await backend.generateProof(witness, { keccakZK: true });
    // JS verification

    const verified = await backend.verifyProof(proofData, { keccakZK: true });
    expect(verified, 'Proof fails verification in JS').to.be.true;

    // Smart contract verification

    // Link the ZKTranscriptLib
    const ZKTranscriptLib = await ethers.deployContract(testInfo.zk_lib);
    await ZKTranscriptLib.waitForDeployment();

    const contract = await ethers.deployContract(testInfo.compiled, [], {
      libraries: {
        ZKTranscriptLib: await ZKTranscriptLib.getAddress(),
      },
    });

    const result = await contract.verify(proofData.proof, proofData.publicInputs);

    expect(result).to.be.true;
  });
});

import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve } from 'path';
import toml from 'toml';

import { PathToFileSourceMap, compile, init_log_level as compilerLogLevel } from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend, flattenPublicInputs } from '@noir-lang/backend_barretenberg';

compilerLogLevel('INFO');

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

    const noirSourcePath = resolve(`${base_relative_path}/${test_case}/src/main.nr`);
    const sourceMap = new PathToFileSourceMap();
    sourceMap.add_source_code(noirSourcePath, readFileSync(noirSourcePath, 'utf-8'));

    const compileResult = compile(noirSourcePath, undefined, undefined, sourceMap);
    if (!('program' in compileResult)) {
      throw new Error('Compilation failed');
    }

    const noir_program = compileResult.program;

    const backend = new BarretenbergBackend(noir_program);
    const program = new Noir(noir_program, backend);

    // JS Proving

    const prover_toml = readFileSync(resolve(`${base_relative_path}/${test_case}/Prover.toml`)).toString();
    const inputs = toml.parse(prover_toml);

    const proofData = await program.generateFinalProof(inputs);

    // JS verification

    const verified = await program.verifyFinalProof(proofData);
    expect(verified, 'Proof fails verification in JS').to.be.true;

    // Smart contract verification

    const contract = await ethers.deployContract(testInfo.compiled, []);

    const result = await contract.verify(proofData.proof, flattenPublicInputs(proofData.publicInputs));

    expect(result).to.be.true;
  });
});

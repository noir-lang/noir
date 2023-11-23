import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve } from 'path';
import toml from 'toml';

import { compile, init_log_level as compilerLogLevel } from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';

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

    const compileResult = compile(noir_source_path);
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

    const contract = await ethers.deployContract(testInfo.compiled, [], {});

    const publicInputIndices = [...proofData.publicInputs.keys()].sort();
    const flattenedPublicInputs = publicInputIndices.map((index) =>
      hexToUint8Array(proofData.publicInputs.get(index) as string),
    );
    const publicInputsConcatenated = flattenUint8Arrays(flattenedPublicInputs);

    const result = await contract.verify(proofData.proof, publicInputsConcatenated);

    expect(result).to.be.true;
  });
});

function flattenUint8Arrays(arrays: Uint8Array[]): Uint8Array {
  const totalLength = arrays.reduce((acc, val) => acc + val.length, 0);
  const result = new Uint8Array(totalLength);

  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }

  return result;
}

function hexToUint8Array(hex: string): Uint8Array {
  const sanitised_hex = BigInt(hex).toString(16).padStart(64, '0');

  const len = sanitised_hex.length / 2;
  const u8 = new Uint8Array(len);

  let i = 0;
  let j = 0;
  while (i < len) {
    u8[i] = parseInt(sanitised_hex.slice(j, j + 2), 16);
    i += 1;
    j += 2;
  }

  return u8;
}

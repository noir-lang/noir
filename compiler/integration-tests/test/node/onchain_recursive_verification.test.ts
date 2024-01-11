import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve, join } from 'path';
import toml from 'toml';

import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { Field, InputMap } from '@noir-lang/noirc_abi';

import { compile, createFileManager } from '@noir-lang/noir_wasm';

it(`smart contract can verify a recursive proof`, async () => {
  const basePath = resolve(join(__dirname, '../../../../'));
  const fm = createFileManager(basePath);
  const innerCompilationResult = await compile(
    fm,
    join(basePath, './test_programs/execution_success/assert_statement'),
  );
  if (!('program' in innerCompilationResult)) {
    throw new Error('Compilation failed');
  }
  const innerProgram = innerCompilationResult.program;

  const recursionCompilationResult = await compile(
    fm,
    join(basePath, './compiler/integration-tests/circuits/recursion'),
  );
  if (!('program' in recursionCompilationResult)) {
    throw new Error('Compilation failed');
  }
  const recursionProgram = recursionCompilationResult.program;

  // Intermediate proof

  const inner_backend = new BarretenbergBackend(innerProgram);
  const inner = new Noir(innerProgram);

  const inner_prover_toml = readFileSync(
    join(basePath, `./test_programs/execution_success/assert_statement/Prover.toml`),
  ).toString();

  const inner_inputs = toml.parse(inner_prover_toml);

  const { witness: main_witness } = await inner.execute(inner_inputs);
  const intermediate_proof = await inner_backend.generateIntermediateProof(main_witness);

  expect(await inner_backend.verifyIntermediateProof(intermediate_proof)).to.be.true;

  const { proofAsFields, vkAsFields, vkHash } = await inner_backend.generateIntermediateProofArtifacts(
    intermediate_proof,
    1, // 1 public input
  );

  // Final proof

  const recursion_backend = new BarretenbergBackend(recursionProgram);
  const recursion = new Noir(recursionProgram, recursion_backend);

  const recursion_inputs: InputMap = {
    verification_key: vkAsFields,
    proof: proofAsFields,
    public_inputs: [inner_inputs.y as Field],
    key_hash: vkHash,
  };

  const recursion_proof = await recursion.generateFinalProof(recursion_inputs);
  expect(await recursion.verifyFinalProof(recursion_proof)).to.be.true;

  // Smart contract verification

  const contract = await ethers.deployContract('contracts/recursion.sol:UltraVerifier', []);

  const result = await contract.verify.staticCall(recursion_proof.proof, recursion_proof.publicInputs);

  expect(result).to.be.true;
});

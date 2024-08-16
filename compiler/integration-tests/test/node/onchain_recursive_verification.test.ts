import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve, join } from 'path';
import toml from 'toml';

import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { Field, InputMap } from '@noir-lang/noirc_abi';

import { compile, createFileManager } from '@noir-lang/noir_wasm';

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/6672): Reinstate this test.
it.skip(`smart contract can verify a recursive proof`, async () => {
  const basePath = resolve(join(__dirname, '../../../../'));
  const fm = createFileManager(basePath);
  const innerCompilationResult = await compile(
    fm,
    join(basePath, './test_programs/execution_success/assert_statement_recursive'),
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
    join(basePath, `./test_programs/execution_success/assert_statement_recursive/Prover.toml`),
  ).toString();

  const inner_inputs = toml.parse(inner_prover_toml);

  const { witness: main_witness } = await inner.execute(inner_inputs);
  const intermediate_proof = await inner_backend.generateProof(main_witness);

  expect(await inner_backend.verifyProof(intermediate_proof)).to.be.true;

  const { proofAsFields, vkAsFields, vkHash } = await inner_backend.generateRecursiveProofArtifacts(
    intermediate_proof,
    1, // 1 public input
  );

  // Final proof

  const recursion = new Noir(recursionProgram);

  const recursion_inputs: InputMap = {
    verification_key: vkAsFields,
    proof: proofAsFields,
    public_inputs: [inner_inputs.y as Field],
    key_hash: vkHash,
  };

  const { witness: recursionWitness } = await recursion.execute(recursion_inputs);

  const recursion_backend = new BarretenbergBackend(recursionProgram);
  const recursion_proof = await recursion_backend.generateProof(recursionWitness);
  expect(await recursion_backend.verifyProof(recursion_proof)).to.be.true;

  // Smart contract verification

  const contract = await ethers.deployContract('contracts/recursion.sol:UltraVerifier', []);

  const result = await contract.verify.staticCall(recursion_proof.proof, recursion_proof.publicInputs);

  expect(result).to.be.true;
});

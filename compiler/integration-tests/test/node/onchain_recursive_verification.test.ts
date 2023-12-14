import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve } from 'path';
import toml from 'toml';

import {
  compile,
  CompiledProgram,
  init_log_level as compilerLogLevel,
  PathToFileSourceMap,
} from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend, flattenPublicInputs } from '@noir-lang/backend_barretenberg';
import { Field, InputMap } from '@noir-lang/noirc_abi';

compilerLogLevel('INFO');

it(`smart contract can verify a recursive proof`, async () => {
  const innerSourcePath = resolve(`../../test_programs/execution_success/assert_statement/src/main.nr`);
  const sourceMapInnerProgram = new PathToFileSourceMap();
  sourceMapInnerProgram.add_source_code(innerSourcePath, readFileSync(innerSourcePath, 'utf-8'));
  const innerProgram = (
    compile(innerSourcePath, undefined, undefined, sourceMapInnerProgram) as { program: CompiledProgram }
  ).program;

  const recursionSourcePath = resolve(`./circuits/recursion/src/main.nr`);
  const sourceMapRecursionProgram = new PathToFileSourceMap();
  sourceMapRecursionProgram.add_source_code(recursionSourcePath, readFileSync(recursionSourcePath, 'utf-8'));
  const recursionProgram = (
    compile(recursionSourcePath, undefined, undefined, sourceMapRecursionProgram) as { program: CompiledProgram }
  ).program;

  // Intermediate proof

  const inner_backend = new BarretenbergBackend(innerProgram);
  const inner = new Noir(innerProgram);

  const inner_prover_toml = readFileSync(
    resolve(`../../test_programs/execution_success/assert_statement/Prover.toml`),
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

  const result = await contract.verify.staticCall(
    recursion_proof.proof,
    flattenPublicInputs(recursion_proof.publicInputs),
  );

  expect(result).to.be.true;
});

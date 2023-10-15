import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve } from 'path';
import toml from 'toml';

import { compile, init_log_level as compilerLogLevel } from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { Field, InputMap } from '@noir-lang/noirc_abi';

compilerLogLevel('INFO');

it(`smart contract can verify a recursive proof`, async () => {
  const main_source_path = resolve(`./circuits/main/src/main.nr`);
  const main_program = compile(main_source_path);

  const recursion_source_path = resolve(`./circuits/recursion/src/main.nr`);
  const recursion_program = compile(recursion_source_path);

  // Intermediate proof

  const main_backend = new BarretenbergBackend(main_program);
  const main = new Noir(main_program);

  const main_prover_toml = readFileSync(resolve(`./circuits/main/Prover.toml`)).toString();
  const main_inputs = toml.parse(main_prover_toml);

  const { witness: main_witness } = await main.execute(main_inputs);
  const intermediate_proof = await main_backend.generateIntermediateProof(main_witness);

  expect(await main_backend.verifyIntermediateProof(intermediate_proof)).to.be.true;

  const { proofAsFields, vkAsFields, vkHash } = await main_backend.generateIntermediateProofArtifacts(
    intermediate_proof,
    1, // 1 public input
  );

  // Final proof

  const recursion_backend = new BarretenbergBackend(recursion_program);
  const recursion = new Noir(recursion_program, recursion_backend);

  const recursion_inputs: InputMap = {
    verification_key: vkAsFields,
    proof: proofAsFields,
    public_inputs: [main_inputs.y as Field],
    key_hash: vkHash,
    input_aggregation_object: ['0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0'],
  };

  const recursion_proof = await recursion.generateFinalProof(recursion_inputs);

  // Smart contract verification

  const contract = await ethers.deployContract('contracts/recursion.sol:UltraVerifier', []);

  const result = await contract.verify.staticCall(recursion_proof.proof, recursion_proof.publicInputs);

  expect(result).to.be.true;
});

import { expect } from 'chai';
import { ethers } from 'hardhat';
import { readFileSync } from 'node:fs';
import { resolve, join } from 'path';
import toml from 'toml';
import { Noir } from '@noir-lang/noir_js';
import { Barretenberg, RawBuffer, UltraHonkBackend } from '@aztec/bb.js';
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

  const innerBackend = new UltraHonkBackend(innerProgram.bytecode, {}, { recursive: true });
  const inner = new Noir(innerProgram);

  const innerProverToml = readFileSync(
    join(basePath, `./test_programs/execution_success/assert_statement/Prover.toml`),
  ).toString();

  const innerInputs = toml.parse(innerProverToml);

  // Generate intermediate proof
  const { witness: main_witness } = await inner.execute(innerInputs);
  const { proof: intermediateProof, publicInputs: intermediatePublicInputs } =
    await innerBackend.generateProofForRecursiveAggregation(main_witness);

  // Get verification key for inner circuit as fields
  const innerCircuitVerificationKey = await innerBackend.getVerificationKey();
  const barretenbergAPI = await Barretenberg.new({ threads: 1 });
  const vkAsFields = await barretenbergAPI.acirVkAsFieldsUltraHonk(new RawBuffer(innerCircuitVerificationKey));

  // Generate proof of the recursive circuit
  const recursiveCircuitNoir = new Noir(recursionProgram);
  const recursiveBackend = new UltraHonkBackend(recursionProgram.bytecode, { threads: 1 });

  const recursiveInputs = {
    proof: intermediateProof,
    public_inputs: intermediatePublicInputs,
    verification_key: vkAsFields.map((field) => field.toString()),
  };
  const { witness: recursiveWitness } = await recursiveCircuitNoir.execute(recursiveInputs);
  const { proof: recursiveProof, publicInputs: recursivePublicInputs } = await recursiveBackend.generateProof(
    recursiveWitness,
    { keccak: true },
  );

  // Verify recursive proof
  const verified = await recursiveBackend.verifyProof(
    { proof: recursiveProof, publicInputs: recursivePublicInputs },
    { keccak: true },
  );

  expect(verified).to.be.true;

  // Smart contract verification
  const contract = await ethers.deployContract('contracts/recursion.sol:UltraVerifier', []);
  const result = await contract.verify.staticCall(recursiveProof, recursivePublicInputs);

  expect(result).to.be.true;
});

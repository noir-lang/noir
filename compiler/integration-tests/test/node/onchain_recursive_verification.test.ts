import { expect } from 'chai';
import { ethers } from 'hardhat';
import { CompiledCircuit, Noir } from '@noir-lang/noir_js';
import { Barretenberg, RawBuffer, UltraHonkBackend } from '@aztec/bb.js';

import assertLtCircuit from '../../circuits/assert_lt/target/assert_lt.json' assert { type: 'json' };
import recursionCircuit from '../../circuits/recursion/target/recursion.json' assert { type: 'json' };

it(`smart contract can verify a recursive proof`, async () => {
  // Inner circuit
  const innerBackend = new UltraHonkBackend(assertLtCircuit.bytecode, {}, { recursive: true });
  const inner = new Noir(assertLtCircuit as CompiledCircuit);
  const innerInputs = {
    x: '2',
    y: '3',
  };

  // Generate intermediate proof
  const { witness: main_witness } = await inner.execute(innerInputs);
  const { proof: intermediateProof, publicInputs: intermediatePublicInputs } =
    await innerBackend.generateProofForRecursiveAggregation(main_witness);

  // Get verification key for inner circuit as fields
  const innerCircuitVerificationKey = await innerBackend.getVerificationKey();
  const barretenbergAPI = await Barretenberg.new({ threads: 1 });
  const vkAsFields = await barretenbergAPI.acirVkAsFieldsUltraHonk(new RawBuffer(innerCircuitVerificationKey));

  // Generate proof of the recursive circuit
  const recursiveCircuitNoir = new Noir(recursionCircuit as CompiledCircuit);
  const recursiveBackend = new UltraHonkBackend(recursionCircuit.bytecode, { threads: 1 });

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
  const contract = await ethers.deployContract('contracts/recursion.sol:HonkVerifier', []);
  const result = await contract.verify.staticCall(recursiveProof, recursivePublicInputs);

  expect(result).to.be.true;
});

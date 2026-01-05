import { expect } from 'chai';
import { ethers } from 'hardhat';
import { CompiledCircuit, Noir } from '@noir-lang/noir_js';
import { Barretenberg, UltraHonkBackend, deflattenFields } from '@aztec/bb.js';

import assertLtCircuit from '../../circuits/assert_lt/target/assert_lt.json' assert { type: 'json' };
import recursionCircuit from '../../circuits/recursion/target/recursion.json' assert { type: 'json' };

it(`smart contract can verify a recursive proof`, async () => {
  const barretenbergAPI = await Barretenberg.new();

  // Inner circuit
  const innerBackend = new UltraHonkBackend(assertLtCircuit.bytecode, barretenbergAPI);
  const inner = new Noir(assertLtCircuit as CompiledCircuit);
  const innerInputs = {
    x: '2',
    y: '3',
  };

  // Generate intermediate proof
  const { witness: main_witness } = await inner.execute(innerInputs);
  const { proof: intermediateProof, publicInputs: intermediatePublicInputs } =
    await innerBackend.generateProof(main_witness);

  // Get verification key for inner circuit as fields
  const innerCircuitVerificationKey = await innerBackend.getVerificationKey();
  const vkAsFields = await barretenbergAPI.vkAsFields({ verificationKey: innerCircuitVerificationKey });
  const vkHash = await barretenbergAPI.poseidon2Hash({ inputs: vkAsFields.fields });

  // Generate proof of the recursive circuit
  const recursiveCircuitNoir = new Noir(recursionCircuit as CompiledCircuit);
  const recursiveBackend = new UltraHonkBackend(recursionCircuit.bytecode, barretenbergAPI);

  const vkAsFieldsReal = vkAsFields.fields.map((field) => {
    let fieldBigint = 0n;
    for (const byte of field) {
      fieldBigint <<= 8n;
      fieldBigint += BigInt(byte);
    }
    return fieldBigint.toString();
  });

  let vkHashBigInt = 0n;
  for (const byte of vkHash.hash) {
    vkHashBigInt <<= 8n;
    vkHashBigInt += BigInt(byte);
  }

  const recursiveInputs = {
    proof: deflattenFields(intermediateProof),
    public_inputs: intermediatePublicInputs,
    verification_key: vkAsFieldsReal,
    key_hash: vkHashBigInt.toString(),
  };
  const { witness: recursiveWitness } = await recursiveCircuitNoir.execute(recursiveInputs);
  const { proof: recursiveProof, publicInputs: recursivePublicInputs } = await recursiveBackend.generateProof(
    recursiveWitness,
    { keccakZK: true },
  );

  // Verify recursive proof
  const verified = await recursiveBackend.verifyProof(
    { proof: recursiveProof, publicInputs: recursivePublicInputs },
    { keccakZK: true },
  );

  expect(verified).to.be.true;

  // Smart contract verification

  // Link the ZKTranscriptLib
  const ZKTranscriptLib = await ethers.deployContract('contracts/recursion.sol:ZKTranscriptLib');
  await ZKTranscriptLib.waitForDeployment();

  const contract = await ethers.deployContract('contracts/recursion.sol:HonkVerifier', [], {
    libraries: {
      ZKTranscriptLib: await ZKTranscriptLib.getAddress(),
    },
  });
  const result = await contract.verify.staticCall(recursiveProof, recursivePublicInputs);

  expect(result).to.be.true;

  await api.destroy();
});

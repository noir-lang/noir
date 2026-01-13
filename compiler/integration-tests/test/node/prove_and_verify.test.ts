import { expect } from 'chai';
import assert_lt_json from '../../circuits/assert_lt/target/assert_lt.json' assert { type: 'json' };
import { Noir } from '@noir-lang/noir_js';
import { UltraHonkVerifierBackend, UltraHonkBackend, Barretenberg } from '@aztec/bb.js';
import { CompiledCircuit } from '@noir-lang/types';

const assert_lt_program = assert_lt_json as CompiledCircuit;

it('end-to-end proof creation and verification', async () => {
  const barretenbergAPI = await Barretenberg.new();
  const honkBackend = new UltraHonkBackend(assert_lt_program.bytecode, barretenbergAPI);

  // Noir.Js part
  const inputs = {
    x: '2',
    y: '3',
  };

  const program = new Noir(assert_lt_program);

  const { witness } = await program.execute(inputs);

  // bb.js part
  //
  // Proof creation
  const proof = await honkBackend.generateProof(witness);

  // Proof verification
  const isValid = await honkBackend.verifyProof(proof);
  expect(isValid).to.be.true;
});

it('end-to-end proof creation and verification -- Verifier API', async () => {
  const barretenbergAPI = await Barretenberg.new();
  const honkBackend = new UltraHonkBackend(assert_lt_program.bytecode, barretenbergAPI);

  // Noir.Js part
  const inputs = {
    x: '2',
    y: '3',
  };

  // Execute program
  const program = new Noir(assert_lt_program);
  const { witness } = await program.execute(inputs);

  // Generate proof
  const proof = await honkBackend.generateProof(witness);

  const verificationKey = await honkBackend.getVerificationKey();

  // Proof verification
  const verifier = new UltraHonkVerifierBackend(barretenbergAPI);
  const isValid = await verifier.verifyProof({ ...proof, verificationKey });
  expect(isValid).to.be.true;
});

it('end-to-end proving and verification with different instances', async () => {
  const barretenbergAPI = await Barretenberg.new();
  const honkBackend = new UltraHonkBackend(assert_lt_program.bytecode, barretenbergAPI);

  // Noir.Js part
  const inputs = {
    x: '2',
    y: '3',
  };

  const program = new Noir(assert_lt_program);

  const { witness } = await program.execute(inputs);

  // bb.js part
  const proof = await honkBackend.generateProof(witness);

  const verifier = new UltraHonkBackend(assert_lt_program.bytecode, barretenbergAPI);
  const proof_is_valid = await verifier.verifyProof(proof);
  expect(proof_is_valid).to.be.true;
});

import { expect } from 'chai';
import assert_lt_json from '../../circuits/assert_lt/target/assert_lt.json' assert { type: 'json' };
import fold_fibonacci_json from '../../circuits/fold_fibonacci/target/fold_fibonacci.json' assert { type: 'json' };
import { Noir } from '@noir-lang/noir_js';
import { UltraHonkVerifierBackend, UltraHonkBackend } from '@aztec/bb.js';
import { CompiledCircuit } from '@noir-lang/types';

const assert_lt_program = assert_lt_json as CompiledCircuit;
const fold_fibonacci_program = fold_fibonacci_json as CompiledCircuit;

const honkBackend = new UltraHonkBackend(assert_lt_program.bytecode);

it('end-to-end proof creation and verification', async () => {
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
  const verifier = new UltraHonkVerifierBackend();
  const isValid = await verifier.verifyProof({ ...proof, verificationKey });
  expect(isValid).to.be.true;
});

it('end-to-end proving and verification with different instances', async () => {
  // Noir.Js part
  const inputs = {
    x: '2',
    y: '3',
  };

  const program = new Noir(assert_lt_program);

  const { witness } = await program.execute(inputs);

  // bb.js part
  const proof = await honkBackend.generateProof(witness);

  const verifier = new UltraHonkBackend(assert_lt_program.bytecode);
  const proof_is_valid = await verifier.verifyProof(proof);
  expect(proof_is_valid).to.be.true;
});

it('end-to-end proof creation and verification for multiple ACIR circuits', async () => {
  // Noir.Js part
  const inputs = {
    x: '10',
  };

  const program = new Noir(fold_fibonacci_program);

  const { witness } = await program.execute(inputs);

  // bb.js part
  //
  // Proof creation
  const honkBackend = new UltraHonkBackend(fold_fibonacci_program.bytecode);
  const proof = await honkBackend.generateProof(witness);

  // Proof verification
  const isValid = await honkBackend.verifyProof(proof);
  expect(isValid).to.be.true;
});

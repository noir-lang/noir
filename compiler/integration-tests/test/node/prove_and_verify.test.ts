import { expect } from 'chai';
import assert_lt_json from '../../circuits/assert_lt/target/assert_lt.json' assert { type: 'json' };
import fold_fibonacci_json from '../../circuits/fold_fibonacci/target/fold_fibonacci.json' assert { type: 'json' };
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergVerifier, UltraPlonkBackend, UltraHonkBackend } from '@aztec/bb.js';
import { CompiledCircuit } from '@noir-lang/types';

const assert_lt_program = assert_lt_json as CompiledCircuit;
const fold_fibonacci_program = fold_fibonacci_json as CompiledCircuit;

const backend = new UltraPlonkBackend(assert_lt_program.bytecode);

it('end-to-end proof creation and verification (outer)', async () => {
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
  const proof = await backend.generateProof(witness);

  // Proof verification
  const isValid = await backend.verifyProof(proof);
  expect(isValid).to.be.true;
});

it('end-to-end proof creation and verification (outer) -- Verifier API', async () => {
  // Noir.Js part
  const inputs = {
    x: '2',
    y: '3',
  };

  // Execute program
  const program = new Noir(assert_lt_program);
  const { witness } = await program.execute(inputs);

  // Generate proof
  const proof = await backend.generateProof(witness);

  const verificationKey = await backend.getVerificationKey();

  // Proof verification
  const verifier = new BarretenbergVerifier();
  const isValid = await verifier.verifyUltraPlonkProof(proof, verificationKey);
  expect(isValid).to.be.true;
});

// TODO: maybe switch to using assert_statement_recursive here to test both options
it('end-to-end proof creation and verification (inner)', async () => {
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
  const proof = await backend.generateProof(witness);

  // Proof verification
  const isValid = await backend.verifyProof(proof);
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
  const proof = await backend.generateProof(witness);

  const verifier = new UltraPlonkBackend(assert_lt_program.bytecode);
  const proof_is_valid = await verifier.verifyProof(proof);
  expect(proof_is_valid).to.be.true;
});

// This bug occurs when we use the same backend to create an inner proof and then an outer proof
// and then try to verify either one of them.
//
// The panic occurs when we try to verify the outer/inner proof that was created.
// If we only create one type of proof, then this works as expected.
//
// If we do not create an inner proof, then this will work as expected.
it('[BUG] -- bb.js null function or function signature mismatch (outer-inner) ', async () => {
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
  //
  // Create a proof using both proving systems, the majority of the time
  // one would only use outer proofs.
  const proofOuter = await backend.generateProof(witness);
  const _proofInner = await backend.generateProof(witness);

  // Proof verification
  //
  const isValidOuter = await backend.verifyProof(proofOuter);
  expect(isValidOuter).to.be.true;
  // We can also try verifying an inner proof and it will fail.
  const isValidInner = await backend.verifyProof(_proofInner);
  expect(isValidInner).to.be.true;
});

it('end-to-end proof creation and verification for multiple ACIR circuits (inner)', async () => {
  // Noir.Js part
  const inputs = {
    x: '10',
  };

  const program = new Noir(fold_fibonacci_program);

  const { witness } = await program.execute(inputs);

  // bb.js part
  //
  // Proof creation
  const backend = new UltraPlonkBackend(fold_fibonacci_program.bytecode);
  const proof = await backend.generateProof(witness);

  // Proof verification
  const isValid = await backend.verifyProof(proof);
  expect(isValid).to.be.true;
});

const honkBackend = new UltraHonkBackend(assert_lt_program.bytecode);

it('UltraHonk end-to-end proof creation and verification (outer)', async () => {
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

it('UltraHonk end-to-end proof creation and verification (outer) -- Verifier API', async () => {
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
  const verifier = new BarretenbergVerifier();
  const isValid = await verifier.verifyUltraHonkProof(proof, verificationKey);
  expect(isValid).to.be.true;
});

it('UltraHonk end-to-end proof creation and verification (inner)', async () => {
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

it('UltraHonk end-to-end proving and verification with different instances', async () => {
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

it('[BUG] -- UltraHonk bb.js null function or function signature mismatch (outer-inner) ', async () => {
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
  //
  // Create a proof using both proving systems, the majority of the time
  // one would only use outer proofs.
  const proofOuter = await honkBackend.generateProof(witness);
  const _proofInner = await honkBackend.generateProof(witness);

  // Proof verification
  //
  const isValidOuter = await honkBackend.verifyProof(proofOuter);
  expect(isValidOuter).to.be.true;
  // We can also try verifying an inner proof and it will fail.
  const isValidInner = await honkBackend.verifyProof(_proofInner);
  expect(isValidInner).to.be.true;
});

it('UltraHonk end-to-end proof creation and verification for multiple ACIR circuits (inner)', async () => {
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

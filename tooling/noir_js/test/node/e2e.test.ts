import { expect } from 'chai';
import assert_lt_json from '../noir_compiled_examples/assert_lt/target/assert_lt.json' assert { type: 'json' };
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend as Backend } from '@noir-lang/backend_barretenberg';
import { CompiledCircuit } from '@noir-lang/types';

const assert_lt_program = assert_lt_json as CompiledCircuit;

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
  const prover = new Backend(assert_lt_program);
  const proof = await prover.generateFinalProof(witness);

  // Proof verification
  const isValid = await prover.verifyFinalProof(proof);
  expect(isValid).to.be.true;
});

it('end-to-end proof creation and verification (outer) -- Program API', async () => {
  // Noir.Js part
  const inputs = {
    x: '2',
    y: '3',
  };

  // Initialize backend
  const backend = new Backend(assert_lt_program);
  // Initialize program
  const program = new Noir(assert_lt_program, backend);
  // Generate proof
  const proof = await program.generateFinalProof(inputs);

  // Proof verification
  const isValid = await program.verifyFinalProof(proof);
  expect(isValid).to.be.true;
});

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
  const prover = new Backend(assert_lt_program);
  const proof = await prover.generateIntermediateProof(witness);

  // Proof verification
  const isValid = await prover.verifyIntermediateProof(proof);
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
  const prover = new Backend(assert_lt_program);

  const proof = await prover.generateFinalProof(witness);

  const verifier = new Backend(assert_lt_program);
  const proof_is_valid = await verifier.verifyFinalProof(proof);
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
  const prover = new Backend(assert_lt_program);
  // Create a proof using both proving systems, the majority of the time
  // one would only use outer proofs.
  const proofOuter = await prover.generateFinalProof(witness);
  const _proofInner = await prover.generateIntermediateProof(witness);

  // Proof verification
  //
  const isValidOuter = await prover.verifyFinalProof(proofOuter);
  expect(isValidOuter).to.be.true;
  // We can also try verifying an inner proof and it will fail.
  const isValidInner = await prover.verifyIntermediateProof(_proofInner);
  expect(isValidInner).to.be.true;
});

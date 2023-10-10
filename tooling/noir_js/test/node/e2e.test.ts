import { expect } from 'chai';
import assert_lt_json from '../noir_compiled_examples/assert_lt/target/assert_lt.json' assert { type: 'json' };
import { Noir } from '../../src/index.js';
import { BarretenbergBackend as Backend } from '@noir-lang/backend_barretenberg';
import { CompiledCircuit } from '@noir-lang/types';

const assert_lt_program = assert_lt_json as CompiledCircuit;

const inputs = {
  x: '2',
  y: '3',
};

describe('Outer proofs', () => {
  let backend: Backend;
  let noir: Noir;

  before(() => {
    backend = new Backend(assert_lt_program, { numOfThreads: 4 });
    noir = new Noir(backend);
  });

  it('Creates and verifies end-to-end outer proofs with underlying backend API', async () => {
    // Noir.Js part
    const serializedWitness = await noir.generateWitness(assert_lt_program, inputs);

    // BackendBarretenberg part
    const prover = new Backend(assert_lt_program, { numOfThreads: 4 });
    const proof = await prover.generateFinalProof(serializedWitness);
    const isValid = await prover.verifyFinalProof(proof);

    // tests
    expect(isValid).to.be.true;
  });

  it('Creates and verifies end-to-end outer proofs with Noir API', async () => {
    const proof = await noir.generateFinalProof(inputs);
    const isValid = await noir.verifyFinalProof(proof);

    // tests
    expect(isValid).to.be.true;
  });
});

describe('Inner proofs', () => {
  let backend: Backend;
  let noir: Noir;

  before(() => {
    backend = new Backend(assert_lt_program, { numOfThreads: 4 });
    noir = new Noir(); // backendless noir;
  });

  it('Creates and verifies end-to-end inner proofs with underlying backend API', async () => {
    // Noir.Js part
    const inputs = {
      x: '2',
      y: '3',
    };
    const serializedWitness = await noir.generateWitness(assert_lt_program, inputs);

    // bb.js part
    //
    // Proof creation
    const proof = await backend.generateIntermediateProof(serializedWitness);

    // Proof verification
    const isValid = await backend.verifyIntermediateProof(proof);
    expect(isValid).to.be.true;
  });

  // The "real" workflow will involve a prover and a verifier on different systems.
  //
  // We cannot do this in our tests because they will panic with:
  // `RuntimeError: null function or function signature mismatch`
  //
  // This happens when we we create a proof with one barretenberg instance and
  // try to verify it with another.
  //
  // If this bug is fixed, we can remove this test and split barretenberg into
  // a prover and verifier class to more accurately reflect what happens in production.
  //
  // If its not fixable, we can leave it in as documentation of this behavior.
  it('Expects the "null function or function signature mismatch" if using different instance', async () => {
    const serializedWitness = await noir.generateWitness(assert_lt_program, inputs);

    // bb.js part
    const proof = await backend.generateFinalProof(serializedWitness);

    try {
      const verifier = new Backend(assert_lt_program);
      await verifier.verifyFinalProof(proof);
      expect.fail(
        'bb.js currently returns a bug when we try to verify a proof with a different Barretenberg instance that created it.',
      );
    } catch (error) {
      const knownError = error as Error;
      expect(knownError.message).to.contain('null function or function signature mismatch');
    }
  });

  // This bug occurs when we use the same backend to create an inner proof and then an outer proof
  // and then try to verify either one of them.
  //
  // The panic occurs when we try to verify the outer/inner proof that was created.
  // If we only create one type of proof, then this works as expected.
  //
  // If we do not create an inner proof, then this will work as expected.
  it('Expects the "null function or function signature mismatch" when mixing different proof types', async () => {
    const serializedWitness = await noir.generateWitness(assert_lt_program, inputs);

    // bb.js part
    //
    // Proof creation
    //
    const prover = new Backend(assert_lt_program);
    // Create a proof using both proving systems, the majority of the time
    // one would only use outer proofs.
    const proofOuter = await prover.generateFinalProof(serializedWitness);
    const _proofInner = await prover.generateIntermediateProof(serializedWitness);

    // Proof verification
    //
    try {
      const isValidOuter = await prover.verifyFinalProof(proofOuter);
      expect(isValidOuter).to.be.true;
      // We can also try verifying an inner proof and it will fail.
      // const isValidInner = await prover.verifyInnerProof(_proofInner);
      // expect(isValidInner).to.be.true;
      expect.fail('bb.js currently returns a bug when we try to verify an inner and outer proof with the same backend');
    } catch (error) {
      const knownError = error as Error;
      expect(knownError.message).to.contain('null function or function signature mismatch');
    }
  });
});

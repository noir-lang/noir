import { expect } from 'chai';
import assert_lt_json from '../noir_compiled_examples/assert_lt/target/assert_lt.json' assert { type: 'json' };
import { generateWitness, witnessMapToUint8Array } from '../../src/index.js';
import { Backend } from '../backend/barretenberg.js';

it('end-to-end proof creation and verification', async () => {
  // Noir.Js part
  const inputs = {
    x: '2',
    y: '3',
  };
  const solvedWitness = await generateWitness(assert_lt_json, inputs);

  // bb.js part
  //
  // Proof creation
  const prover = new Backend(assert_lt_json.bytecode);
  await prover.init();
  const serializedWitness = witnessMapToUint8Array(solvedWitness);
  const proof = await prover.generateOuterProof(serializedWitness);

  // Proof verification
  const isValid = await prover.verifyOuterProof(proof);
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
it('[BUG] -- bb.js null function or function signature mismatch ', async () => {
  // Noir.Js part
  const inputs = {
    x: '2',
    y: '3',
  };
  const solvedWitness = await generateWitness(assert_lt_json, inputs);

  // bb.js part
  const prover = new Backend(assert_lt_json.bytecode);
  await prover.init();

  const serializedWitness = witnessMapToUint8Array(solvedWitness);
  const proof = await prover.generateOuterProof(serializedWitness);

  try {
    const verifier = new Backend(assert_lt_json.bytecode);
    await verifier.init();
    await verifier.verifyOuterProof(proof);
    expect.fail(
      'bb.js currently returns a bug when we try to verify a proof with a different Barretenberg instance that created it.',
    );
  } catch (error) {
    const knownError = error as Error;
    expect(knownError.message).to.contain('null function or function signature mismatch');
  }
});

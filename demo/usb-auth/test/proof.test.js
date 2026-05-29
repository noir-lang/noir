import { strict as assert } from 'node:assert';
import { createAuthInputs, generateAndVerifyProof } from '../src/proof.js';
import { compileCircuitFromFiles } from '../src/compile-circuit.js';

describe('Noir USB auth proof', () => {
  it('generates and verifies a proof for a valid encrypted-device secret flow', async () => {
    const circuit = await compileCircuitFromFiles(new URL('../', import.meta.url));
    const authInputs = await createAuthInputs({
      deviceSecret: '11',
      userId: 'demo-user',
      challenge: '19',
    });

    const result = await generateAndVerifyProof(circuit, authInputs);

    assert.equal(result.verified, true);
    assert.equal(result.nullifier, authInputs.publicInputs.expected_nullifier);
  });
});

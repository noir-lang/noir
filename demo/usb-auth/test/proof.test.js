import { strict as assert } from 'node:assert';
import { createAuthInputs, generateAndVerifyProof } from '../src/proof.js';
import circuit from '../src/circuit-artifact.js';

describe('Noir USB auth proof', () => {
  it('generates and verifies a proof for a valid encrypted-device secret flow', async () => {
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

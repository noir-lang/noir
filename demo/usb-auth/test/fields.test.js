import { strict as assert } from 'node:assert';
import { computeCommitment, computeNullifier, fieldToString, userIdToField } from '../src/fields.js';

describe('field helpers', () => {
  it('computes the commitment and nullifier formulas used by the Noir circuit', async () => {
    const userIdHash = await userIdToField('demo-user');

    assert.equal(computeCommitment('3', userIdHash), fieldToString(9n + BigInt(userIdHash)));
    assert.equal(computeNullifier('3', '5', userIdHash), fieldToString(15n + BigInt(userIdHash)));
  });
});

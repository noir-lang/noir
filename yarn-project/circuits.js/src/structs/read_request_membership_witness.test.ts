import { makeReadRequestMembershipWitness } from '../tests/factories.js';
import { ReadRequestMembershipWitness } from './read_request_membership_witness.js';

describe('ReadRequestMembershipWitness', () => {
  it('Data after serialization and deserialization is equal to the original', () => {
    const original = makeReadRequestMembershipWitness(0);
    const buf = original.toBuffer();
    const afterSerialization = ReadRequestMembershipWitness.fromBuffer(buf);
    expect(original).toEqual(afterSerialization);
  });
});

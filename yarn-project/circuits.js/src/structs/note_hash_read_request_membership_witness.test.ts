import { makeNoteHashReadRequestMembershipWitness } from '../tests/factories.js';
import { NoteHashReadRequestMembershipWitness } from './note_hash_read_request_membership_witness.js';

describe('NoteHashReadRequestMembershipWitness', () => {
  it('Data after serialization and deserialization is equal to the original', () => {
    const original = makeNoteHashReadRequestMembershipWitness(0);
    const buf = original.toBuffer();
    const afterSerialization = NoteHashReadRequestMembershipWitness.fromBuffer(buf);
    expect(original).toEqual(afterSerialization);
  });
});

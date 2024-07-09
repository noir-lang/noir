import { Fr, GrumpkinScalar } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { NoteSelector } from '@aztec/foundation/abi';
import { updateInlineTestData } from '@aztec/foundation/testing';

import { Note } from '../payload.js';
import { EncryptedNoteLogIncomingBody } from './encrypted_note_log_incoming_body.js';

describe('encrypt log incoming body', () => {
  let grumpkin: Grumpkin;

  beforeAll(() => {
    grumpkin = new Grumpkin();
  });

  it('encrypt and decrypt a note log incoming body', () => {
    const ephSecretKey = GrumpkinScalar.random();
    const viewingSecretKey = GrumpkinScalar.random();

    const ephPubKey = grumpkin.mul(Grumpkin.generator, ephSecretKey);
    const viewingPubKey = grumpkin.mul(Grumpkin.generator, viewingSecretKey);

    const note = Note.random();
    const storageSlot = Fr.random();
    const noteTypeId = NoteSelector.random();

    const body = new EncryptedNoteLogIncomingBody(storageSlot, noteTypeId, note);

    const encrypted = body.computeCiphertext(ephSecretKey, viewingPubKey);

    const recreated = EncryptedNoteLogIncomingBody.fromCiphertext(encrypted, viewingSecretKey, ephPubKey);

    expect(recreated.toBuffer()).toEqual(body.toBuffer());
  });

  it('encrypt a note log incoming body, generate input for noir test', () => {
    // The following 2 are arbitrary fixed values - fixed in order to test a match with Noir
    const ephSecretKey = new GrumpkinScalar(0x23b3127c127b1f29a7adff5cccf8fb06649e7ca01d9de27b21624098b897babdn);
    const viewingSecretKey = new GrumpkinScalar(0x1fdd0dd8c99b21af8e00d2d130bdc263b36dadcbea84ac5ec9293a0660deca01n);

    const viewingPubKey = grumpkin.mul(Grumpkin.generator, viewingSecretKey);

    const note = new Note([new Fr(1), new Fr(2), new Fr(3)]);
    const noteTypeId = new NoteSelector(1);
    const storageSlot = new Fr(2);

    const body = new EncryptedNoteLogIncomingBody(storageSlot, noteTypeId, note);

    const encrypted = body.computeCiphertext(ephSecretKey, viewingPubKey);

    const byteArrayString = `[${encrypted
      .toString('hex')
      .match(/.{1,2}/g)!
      .map(byte => parseInt(byte, 16))}]`;

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/encrypted_logs/incoming_body.nr',
      'expected_note_body_ciphertext',
      byteArrayString,
    );
  });
});

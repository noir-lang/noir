import { Fr, type GrumpkinScalar, type PublicKey } from '@aztec/circuits.js';
import { NoteSelector } from '@aztec/foundation/abi';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { Note } from '../payload.js';
import { EncryptedLogIncomingBody } from './encrypted_log_incoming_body.js';

export class EncryptedNoteLogIncomingBody extends EncryptedLogIncomingBody {
  constructor(public storageSlot: Fr, public noteTypeId: NoteSelector, public note: Note) {
    super();
  }

  /**
   * Serializes the log body to a buffer WITHOUT the length of the note buffer
   *
   * @returns The serialized log body
   */
  public toBuffer(): Buffer {
    const noteBufferWithoutLength = this.note.toBuffer().subarray(4);
    // Note: We serialize note type to field first because that's how it's done in Noir
    return serializeToBuffer(this.storageSlot, this.noteTypeId.toField(), noteBufferWithoutLength);
  }

  /**
   * Deserialized the log body from a buffer WITHOUT the length of the note buffer
   *
   * @param buf - The buffer to deserialize
   * @returns The deserialized log body
   */
  public static fromBuffer(buf: Buffer): EncryptedNoteLogIncomingBody {
    const reader = BufferReader.asReader(buf);
    const storageSlot = Fr.fromBuffer(reader);
    const noteTypeId = NoteSelector.fromField(Fr.fromBuffer(reader));

    // 2 Fields (storage slot and note type id) are not included in the note buffer
    const fieldsInNote = reader.getLength() / 32 - 2;
    const note = new Note(reader.readArray(fieldsInNote, Fr));

    return new EncryptedNoteLogIncomingBody(storageSlot, noteTypeId, note);
  }

  /**
   * Decrypts a log body
   *
   * @param ciphertext - The ciphertext buffer
   * @param ivskAppOrEphSk - The private key matching the public key used in encryption (the viewing key secret or)
   * @param ephPkOrIvpkApp - The public key generated with the ephemeral secret key used in encryption
   *
   * The "odd" input stems from ivskApp * ephPk == ivpkApp * ephSk producing the same value.
   * This is used to allow for the same decryption function to be used by both the sender and the recipient.
   *
   * @returns The decrypted log body
   */
  public static fromCiphertext(
    ciphertext: Buffer | bigint[],
    ivskAppOrEphSk: GrumpkinScalar,
    ephPkOrIvpkApp: PublicKey,
  ): EncryptedNoteLogIncomingBody {
    const buffer = super.fromCiphertextToBuffer(ciphertext, ivskAppOrEphSk, ephPkOrIvpkApp);
    return EncryptedNoteLogIncomingBody.fromBuffer(buffer);
  }
}

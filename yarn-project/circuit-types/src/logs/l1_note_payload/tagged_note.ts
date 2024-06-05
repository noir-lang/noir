import { AztecAddress, type GrumpkinPrivateKey, type KeyValidationRequest, type PublicKey } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { L1NotePayload } from './l1_note_payload.js';

// placeholder value until tagging is implemented
const PLACEHOLDER_TAG = new Fr(33);

/**
 * Encrypted note payload with a tag used for retrieval by clients.
 */
export class TaggedNote {
  constructor(
    public notePayload: L1NotePayload,
    public incomingTag = PLACEHOLDER_TAG,
    public outgoingTag = PLACEHOLDER_TAG,
  ) {}

  /**
   * Deserializes the TaggedNote object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of TaggedNote.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TaggedNote {
    const reader = BufferReader.asReader(buffer);
    const incomingTag = Fr.fromBuffer(reader);
    const outgoingTag = Fr.fromBuffer(reader);
    const payload = L1NotePayload.fromBuffer(reader);
    return new TaggedNote(payload, incomingTag, outgoingTag);
  }

  /**
   * Serializes the TaggedNote object into a Buffer.
   * @returns Buffer representation of the TaggedNote object (unencrypted).
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(this.incomingTag, this.outgoingTag, this.notePayload);
  }

  /**
   * Create a random TaggedNote (useful for testing purposes).
   * @param contract - The address of a contract the note was emitted from.
   * @returns A random TaggedNote object.
   */
  static random(contract = AztecAddress.random()): TaggedNote {
    return new TaggedNote(L1NotePayload.random(contract));
  }

  public encrypt(
    ephSk: GrumpkinPrivateKey,
    recipient: AztecAddress,
    ivpk: PublicKey,
    ovKeys: KeyValidationRequest,
  ): Buffer {
    return serializeToBuffer(
      this.incomingTag,
      this.outgoingTag,
      this.notePayload.encrypt(ephSk, recipient, ivpk, ovKeys),
    );
  }

  static decryptAsIncoming(data: Buffer | bigint[], ivsk: GrumpkinPrivateKey) {
    // Right now heavily abusing that we will likely fail if bad decryption
    // as some field will likely end up not being in the field etc.
    try {
      const input = Buffer.isBuffer(data) ? data : Buffer.from(data.map((x: bigint) => Number(x)));
      const reader = BufferReader.asReader(input);
      const incomingTag = Fr.fromBuffer(reader);
      const outgoingTag = Fr.fromBuffer(reader);
      const payload = L1NotePayload.decryptAsIncoming(reader.readToEnd(), ivsk);
      return new TaggedNote(payload, incomingTag, outgoingTag);
    } catch (e) {
      return;
    }
  }

  static decryptAsOutgoing(data: Buffer | bigint[], ovsk: GrumpkinPrivateKey) {
    // Right now heavily abusing that we will likely fail if bad decryption
    // as some field will likely end up not being in the field etc.
    try {
      const input = Buffer.isBuffer(data) ? data : Buffer.from(data.map((x: bigint) => Number(x)));
      const reader = BufferReader.asReader(input);
      const incomingTag = Fr.fromBuffer(reader);
      const outgoingTag = Fr.fromBuffer(reader);
      const payload = L1NotePayload.decryptAsOutgoing(reader.readToEnd(), ovsk);
      return new TaggedNote(payload, incomingTag, outgoingTag);
    } catch (e) {
      return;
    }
  }
}

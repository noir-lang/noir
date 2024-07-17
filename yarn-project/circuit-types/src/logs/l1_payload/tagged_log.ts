import { AztecAddress, type GrumpkinScalar, type KeyValidationRequest, type PublicKey } from '@aztec/circuits.js';
import { Fr, NotOnCurveError } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { type EncryptedL2Log } from '../encrypted_l2_log.js';
import { L1EventPayload } from './l1_event_payload.js';
import { L1NotePayload } from './l1_note_payload.js';

// placeholder value until tagging is implemented
const PLACEHOLDER_TAG = new Fr(33);

/**
 * Encrypted log payload with a tag used for retrieval by clients.
 */
export class TaggedLog<Payload extends L1NotePayload | L1EventPayload> {
  constructor(public payload: Payload, public incomingTag = PLACEHOLDER_TAG, public outgoingTag = PLACEHOLDER_TAG) {}

  /**
   * Deserializes the TaggedLog object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of TaggedLog.
   */
  static fromBuffer(buffer: Buffer | BufferReader, payloadType: typeof L1EventPayload): TaggedLog<L1EventPayload>;
  static fromBuffer(buffer: Buffer | BufferReader, payloadType?: typeof L1NotePayload): TaggedLog<L1NotePayload>;
  static fromBuffer(
    buffer: Buffer | BufferReader,
    payloadType: typeof L1NotePayload | typeof L1EventPayload = L1NotePayload,
  ) {
    const reader = BufferReader.asReader(buffer);
    const incomingTag = Fr.fromBuffer(reader);
    const outgoingTag = Fr.fromBuffer(reader);
    const payload =
      payloadType === L1NotePayload ? L1NotePayload.fromBuffer(reader) : L1EventPayload.fromBuffer(reader);

    return new TaggedLog(payload, incomingTag, outgoingTag);
  }

  /**
   * Serializes the TaggedLog object into a Buffer.
   * @returns Buffer representation of the TaggedLog object (unencrypted).
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(this.incomingTag, this.outgoingTag, this.payload);
  }

  static random(payloadType?: typeof L1NotePayload, contract?: AztecAddress): TaggedLog<L1NotePayload>;
  static random(payloadType: typeof L1EventPayload): TaggedLog<L1EventPayload>;
  static random(
    payloadType: typeof L1NotePayload | typeof L1EventPayload = L1NotePayload,
    contract = AztecAddress.random(),
  ): TaggedLog<L1NotePayload | L1EventPayload> {
    return payloadType === L1NotePayload
      ? new TaggedLog(L1NotePayload.random(contract))
      : new TaggedLog(L1EventPayload.random());
  }

  public encrypt(
    ephSk: GrumpkinScalar,
    recipient: AztecAddress,
    ivpk: PublicKey,
    ovKeys: KeyValidationRequest,
  ): Buffer {
    return serializeToBuffer(this.incomingTag, this.outgoingTag, this.payload.encrypt(ephSk, recipient, ivpk, ovKeys));
  }

  static decryptAsIncoming(
    encryptedLog: EncryptedL2Log,
    ivsk: GrumpkinScalar,
    payloadType: typeof L1EventPayload,
  ): TaggedLog<L1EventPayload> | undefined;
  static decryptAsIncoming(
    data: Buffer | bigint[],
    ivsk: GrumpkinScalar,
    payloadType?: typeof L1NotePayload,
  ): TaggedLog<L1NotePayload> | undefined;
  static decryptAsIncoming(
    data: Buffer | bigint[] | EncryptedL2Log,
    ivsk: GrumpkinScalar,
    payloadType: typeof L1NotePayload | typeof L1EventPayload = L1NotePayload,
  ): TaggedLog<L1NotePayload | L1EventPayload> | undefined {
    try {
      if (payloadType === L1EventPayload) {
        const reader = BufferReader.asReader((data as EncryptedL2Log).data);
        const incomingTag = Fr.fromBuffer(reader);
        const outgoingTag = Fr.fromBuffer(reader);
        // We must pass the entire encrypted log in. The tags are not stripped here from the original data
        const payload = L1EventPayload.decryptAsIncoming(data as EncryptedL2Log, ivsk);
        return new TaggedLog(payload, incomingTag, outgoingTag);
      } else {
        const input = Buffer.isBuffer(data) ? data : Buffer.from((data as bigint[]).map((x: bigint) => Number(x)));
        const reader = BufferReader.asReader(input);
        const incomingTag = Fr.fromBuffer(reader);
        const outgoingTag = Fr.fromBuffer(reader);
        const payload = L1NotePayload.decryptAsIncoming(reader.readToEnd(), ivsk);
        return new TaggedLog(payload, incomingTag, outgoingTag);
      }
    } catch (e: any) {
      // Following error messages are expected to occur when decryption fails
      if (
        !(e instanceof NotOnCurveError) &&
        !e.message.endsWith('is greater or equal to field modulus.') &&
        !e.message.startsWith('Invalid AztecAddress length') &&
        !e.message.startsWith('Selector must fit in') &&
        !e.message.startsWith('Attempted to read beyond buffer length')
      ) {
        // If we encounter an unexpected error, we rethrow it
        throw e;
      }
      return;
    }
  }

  static decryptAsOutgoing(
    encryptedLog: EncryptedL2Log,
    ivsk: GrumpkinScalar,
    payloadType: typeof L1EventPayload,
  ): TaggedLog<L1EventPayload> | undefined;
  static decryptAsOutgoing(
    data: Buffer | bigint[],
    ivsk: GrumpkinScalar,
    payloadType?: typeof L1NotePayload,
  ): TaggedLog<L1NotePayload> | undefined;
  static decryptAsOutgoing(
    data: Buffer | bigint[] | EncryptedL2Log,
    ovsk: GrumpkinScalar,
    payloadType: typeof L1NotePayload | typeof L1EventPayload = L1NotePayload,
  ) {
    try {
      if (payloadType === L1EventPayload) {
        const reader = BufferReader.asReader((data as EncryptedL2Log).data);
        const incomingTag = Fr.fromBuffer(reader);
        const outgoingTag = Fr.fromBuffer(reader);
        const payload = L1EventPayload.decryptAsOutgoing(data as EncryptedL2Log, ovsk);
        return new TaggedLog(payload, incomingTag, outgoingTag);
      } else {
        const input = Buffer.isBuffer(data) ? data : Buffer.from((data as bigint[]).map((x: bigint) => Number(x)));
        const reader = BufferReader.asReader(input);
        const incomingTag = Fr.fromBuffer(reader);
        const outgoingTag = Fr.fromBuffer(reader);
        const payload = L1NotePayload.decryptAsOutgoing(reader.readToEnd(), ovsk);
        return new TaggedLog(payload, incomingTag, outgoingTag);
      }
    } catch (e: any) {
      // Following error messages are expected to occur when decryption fails
      if (
        !(e instanceof NotOnCurveError) &&
        !e.message.endsWith('is greater or equal to field modulus.') &&
        !e.message.startsWith('Invalid AztecAddress length') &&
        !e.message.startsWith('Selector must fit in') &&
        !e.message.startsWith('Attempted to read beyond buffer length')
      ) {
        // If we encounter an unexpected error, we rethrow it
        throw e;
      }
      return;
    }
  }
}

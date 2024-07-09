import { Fr, type GrumpkinScalar, type PublicKey } from '@aztec/circuits.js';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { Event } from '../payload.js';
import { EncryptedLogIncomingBody } from './encrypted_log_incoming_body.js';

export class EncryptedEventLogIncomingBody extends EncryptedLogIncomingBody {
  constructor(public randomness: Fr, public eventTypeId: Fr, public event: Event) {
    super();
  }

  /**
   * Serializes the log body to a buffer WITHOUT the length of the event buffer
   *
   * @returns The serialized log body
   */
  public toBuffer(): Buffer {
    const eventBufferWithoutLength = this.event.toBuffer().subarray(4);
    return serializeToBuffer(this.randomness, this.eventTypeId, eventBufferWithoutLength);
  }

  /**
   * Deserialized the log body from a buffer WITHOUT the length of the event buffer
   *
   * @param buf - The buffer to deserialize
   * @returns The deserialized log body
   */
  public static fromBuffer(buf: Buffer): EncryptedEventLogIncomingBody {
    const reader = BufferReader.asReader(buf);
    const randomness = Fr.fromBuffer(reader);
    const eventTypeId = Fr.fromBuffer(reader);

    // 2 Fields (randomness and event type id) are not included in the event buffer
    const fieldsInEvent = reader.getLength() / 32 - 2;
    const event = new Event(reader.readArray(fieldsInEvent, Fr));

    return new EncryptedEventLogIncomingBody(randomness, eventTypeId, event);
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
  ): EncryptedEventLogIncomingBody {
    const buffer = super.fromCiphertextToBuffer(ciphertext, ivskAppOrEphSk, ephPkOrIvpkApp);
    return EncryptedEventLogIncomingBody.fromBuffer(buffer);
  }
}

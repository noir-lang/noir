import { Fr, type GrumpkinPrivateKey, type PublicKey } from '@aztec/circuits.js';
import { Aes128 } from '@aztec/circuits.js/barretenberg';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { Note, deriveAESSecret } from './l1_note_payload/index.js';

export class EncryptedLogBody {
  constructor(public storageSlot: Fr, public noteTypeId: Fr, public note: Note) {}

  /**
   * Serializes the log body to a buffer WITHOUT the length of the note buffer
   *
   * @returns The serialized log body
   */
  public toBuffer(): Buffer {
    const noteBufferWithoutLength = this.note.toBuffer().subarray(4);
    return serializeToBuffer(this.storageSlot, this.noteTypeId, noteBufferWithoutLength);
  }

  /**
   * Deserialized the log body from a buffer WITHOUT the length of the note buffer
   *
   * @param buf - The buffer to deserialize
   * @returns The deserialized log body
   */
  public static fromBuffer(buf: Buffer): EncryptedLogBody {
    const reader = BufferReader.asReader(buf);
    const storageSlot = Fr.fromBuffer(reader);
    const noteTypeId = Fr.fromBuffer(reader);

    // 2 Fields (storage slot and note type id) are not included in the note buffer
    const fieldsInNote = reader.getLength() / 32 - 2;
    const note = new Note(reader.readArray(fieldsInNote, Fr));

    return new EncryptedLogBody(storageSlot, noteTypeId, note);
  }

  /**
   * Encrypts a log body
   *
   * @param secret - The ephemeral secret key
   * @param publicKey - The incoming viewing key for the recipient of this log
   *
   * @returns The ciphertext of the encrypted log body
   */
  public computeCiphertext(secret: GrumpkinPrivateKey, publicKey: PublicKey) {
    const aesSecret = deriveAESSecret(secret, publicKey);
    const key = aesSecret.subarray(0, 16);
    const iv = aesSecret.subarray(16, 32);

    const aes128 = new Aes128();
    const buffer = this.toBuffer();

    return aes128.encryptBufferCBC(buffer, iv, key);
  }

  /**
   * Decrypts a log body
   *
   * @param ciphertext - The ciphertext buffer
   * @param secret - The private key matching the public key used in encryption (the viewing key secret)
   * @param publicKey - The public key generated with the ephemeral secret key used in encryption
   *
   * @returns The decrypted log body
   */
  public static fromCiphertext(
    ciphertext: Buffer | bigint[],
    secret: GrumpkinPrivateKey,
    publicKey: PublicKey,
  ): EncryptedLogBody {
    const input = Buffer.isBuffer(ciphertext) ? ciphertext : Buffer.from(ciphertext.map((x: bigint) => Number(x)));

    const aesSecret = deriveAESSecret(secret, publicKey);
    const key = aesSecret.subarray(0, 16);
    const iv = aesSecret.subarray(16, 32);

    const aes128 = new Aes128();
    const buffer = aes128.decryptBufferCBC(input, iv, key);
    return EncryptedLogBody.fromBuffer(buffer);
  }
}

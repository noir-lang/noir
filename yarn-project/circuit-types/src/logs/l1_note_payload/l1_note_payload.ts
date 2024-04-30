import { AztecAddress, type GrumpkinPrivateKey, type PublicKey } from '@aztec/circuits.js';
import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { decryptBuffer, encryptBuffer } from './encrypt_buffer.js';
import { Note } from './note.js';

/**
 * A class which wraps note data which is pushed on L1.
 * @remarks This data is required to compute a nullifier/to spend a note. Along with that this class contains
 * the necessary functionality to encrypt and decrypt the data.
 */
export class L1NotePayload {
  constructor(
    /**
     * A note as emitted from Noir contract. Can be used along with private key to compute nullifier.
     */
    public note: Note,
    /**
     * Address of the contract this tx is interacting with.
     */
    public contractAddress: AztecAddress,
    /**
     * Storage slot of the contract this tx is interacting with.
     */
    public storageSlot: Fr,
    /**
     * Type identifier for the underlying note, required to determine how to compute its hash and nullifier.
     */
    public noteTypeId: Fr,
  ) {}

  /**
   * Deserializes the L1NotePayload object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of L1NotePayload.
   */
  static fromBuffer(buffer: Buffer | BufferReader): L1NotePayload {
    const reader = BufferReader.asReader(buffer);
    return new L1NotePayload(
      reader.readObject(Note),
      reader.readObject(AztecAddress),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  /**
   * Serializes the L1NotePayload object into a Buffer.
   * @returns Buffer representation of the L1NotePayload object.
   */
  toBuffer() {
    return serializeToBuffer([this.note, this.contractAddress, this.storageSlot, this.noteTypeId]);
  }

  /**
   * Encrypt the L1NotePayload object using the owner's public key and the ephemeral private key.
   * @param incomingViewingPubKey - Public key of the owner of the L1NotePayload object.
   * @returns The encrypted L1NotePayload object.
   */
  public toEncryptedBuffer(incomingViewingPubKey: PublicKey): Buffer {
    const ephSecretKey: GrumpkinPrivateKey = GrumpkinScalar.random();
    return encryptBuffer(this.toBuffer(), ephSecretKey, incomingViewingPubKey);
  }

  /**
   * Decrypts the L1NotePayload object using the owner's incoming viewing secret key.
   * @param data - Encrypted L1NotePayload object.
   * @param incomingViewingSecretKey - Incoming viewing secret key of the owner of the L1NotePayload object.
   * @returns Instance of L1NotePayload if the decryption was successful, undefined otherwise.
   */
  static fromEncryptedBuffer(data: Buffer, incomingViewingSecretKey: GrumpkinPrivateKey): L1NotePayload | undefined {
    const buf = decryptBuffer(data, incomingViewingSecretKey);
    if (!buf) {
      return;
    }
    return L1NotePayload.fromBuffer(buf);
  }

  /**
   * Create a random L1NotePayload object (useful for testing purposes).
   * @returns A random L1NotePayload object.
   */
  static random() {
    return new L1NotePayload(Note.random(), AztecAddress.random(), Fr.random(), Fr.random());
  }
}

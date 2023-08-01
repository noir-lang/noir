import { AztecAddress, PrivateKey, PublicKey } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { serializeToBuffer } from '@aztec/circuits.js/utils';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { decryptBuffer, encryptBuffer } from './encrypt_buffer.js';
import { NotePreimage } from './note_preimage.js';

/**
 * A class which wraps the data required to compute a nullifier/to spend a note. Along with that this class contains
 * the necessary functionality to encrypt and decrypt the data.
 */
export class NoteSpendingInfo {
  constructor(
    /**
     * Preimage which can be used along with private key to compute nullifier.
     */
    public notePreimage: NotePreimage,
    /**
     * Address of the contract this tx is interacting with.
     */
    public contractAddress: AztecAddress,
    /**
     * Address of the owner of the note.
     */
    public ownerAddress: AztecAddress,
    /**
     * Storage slot of the contract this tx is interacting with.
     */
    public storageSlot: Fr,
  ) {}

  /**
   * Deserializes the NoteSpendingInfo object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of NoteSpendingInfo.
   */
  static fromBuffer(buffer: Buffer | BufferReader): NoteSpendingInfo {
    const reader = BufferReader.asReader(buffer);
    return new NoteSpendingInfo(
      reader.readObject(NotePreimage),
      reader.readObject(AztecAddress),
      reader.readObject(AztecAddress),
      reader.readFr(),
    );
  }

  /**
   * Serializes the NoteSpendingInfo object into a Buffer.
   * @returns Buffer representation of the NoteSpendingInfo object.
   */
  toBuffer() {
    return serializeToBuffer([this.notePreimage, this.contractAddress, this.ownerAddress, this.storageSlot]);
  }

  /**
   * Encrypt the NoteSpendingInfo object using the owner's public key and the ephemeral private key.
   * @param ownerPubKey - Public key of the owner of the NoteSpendingInfo object.
   * @param curve - The curve instance to use.
   * @returns The encrypted NoteSpendingInfo object.
   */
  public toEncryptedBuffer(ownerPubKey: PublicKey, curve: Grumpkin): Buffer {
    const ephPrivKey = PrivateKey.random();
    return encryptBuffer(this.toBuffer(), ownerPubKey, ephPrivKey, curve);
  }

  /**
   * Decrypts the NoteSpendingInfo object using the owner's private key.
   * @param data - Encrypted NoteSpendingInfo object.
   * @param ownerPrivKey - Private key of the owner of the NoteSpendingInfo object.
   * @param curve - The curve instance to use.
   * @returns Instance of NoteSpendingInfo if the decryption was successful, undefined otherwise.
   */
  static fromEncryptedBuffer(data: Buffer, ownerPrivKey: PrivateKey, curve: Grumpkin): NoteSpendingInfo | undefined {
    const buf = decryptBuffer(data, ownerPrivKey, curve);
    if (!buf) {
      return;
    }
    return NoteSpendingInfo.fromBuffer(buf);
  }

  /**
   * Create a random NoteSpendingInfo object (useful for testing purposes).
   * @returns A random NoteSpendingInfo object.
   */
  static random() {
    return new NoteSpendingInfo(NotePreimage.random(), AztecAddress.random(), AztecAddress.random(), Fr.random());
  }
}

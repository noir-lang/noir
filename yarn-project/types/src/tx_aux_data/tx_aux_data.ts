import { Fr, Point } from '@aztec/foundation/fields';
import { AztecAddress } from '@aztec/circuits.js';
import { BufferReader } from '@aztec/foundation/serialize';
import { NotePreimage } from './note_preimage.js';
import { serializeToBuffer } from '@aztec/circuits.js/utils';
import { decryptBuffer, encryptBuffer } from './encrypt_buffer.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { randomBytes } from '@aztec/foundation/crypto';

/**
 * A class which wraps the data required to compute a nullifier. Along with that this class contains the necessary
 * functionality to encrypt and decrypt the data.
 */
export class TxAuxData {
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
     * Storage slot of the contract this tx is interacting with.
     */
    public storageSlot: Fr,
  ) {}

  /**
   * Deserializes the TxAuxData object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of TxAuxData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxAuxData {
    const reader = BufferReader.asReader(buffer);
    return new TxAuxData(reader.readObject(NotePreimage), reader.readObject(AztecAddress), reader.readFr());
  }

  /**
   * Serializes the TxAuxData object into a Buffer.
   * @returns Buffer representation of the TxAuxData object.
   */
  toBuffer() {
    return serializeToBuffer([this.notePreimage, this.contractAddress, this.storageSlot]);
  }

  /**
   * Encrypt the TxAuxData object using the owner's public key and the ephemeral private key.
   * @param ownerPubKey - Public key of the owner of the TxAuxData object.
   * @param grumpkin - Arbitrary Grumpkin class instance.
   * @returns The encrypted TxAuxData object.
   */
  public toEncryptedBuffer(ownerPubKey: Point, grumpkin: Grumpkin): Buffer {
    const ephPrivKey = randomBytes(32);
    return encryptBuffer(this.toBuffer(), ownerPubKey, ephPrivKey, grumpkin);
  }

  /**
   * Decrypts the TxAuxData object using the owner's private key.
   * @param data - Encrypted TxAuxData object.
   * @param ownerPrivKey - Private key of the owner of the TxAuxData object.
   * @param grumpkin - Arbitrary Grumpkin class instance.
   * @returns Instance of TxAuxData if the decryption was successful, undefined otherwise.
   */
  static fromEncryptedBuffer(data: Buffer, ownerPrivKey: Buffer, grumpkin: Grumpkin): TxAuxData | undefined {
    const buf = decryptBuffer(data, ownerPrivKey, grumpkin);
    if (!buf) {
      return;
    }
    return TxAuxData.fromBuffer(buf);
  }

  /**
   * Create a random TxAuxData object (useful for testing purposes).
   * @returns A random TxAuxData object.
   */
  static random() {
    return new TxAuxData(NotePreimage.random(), AztecAddress.random(), Fr.random());
  }
}

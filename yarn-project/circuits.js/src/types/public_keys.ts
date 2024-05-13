import { poseidon2Hash } from '@aztec/foundation/crypto';
import { type Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { GeneratorIndex } from '../constants.gen.js';
import { type PublicKey } from './public_key.js';

export class PublicKeys {
  public constructor(
    /** Contract address (typically of an account contract) */
    /** Master nullifier public key */
    public masterNullifierPublicKey: PublicKey,
    /** Master incoming viewing public key */
    public masterIncomingViewingPublicKey: PublicKey,
    /** Master outgoing viewing public key */
    public masterOutgoingViewingPublicKey: PublicKey,
    /** Master tagging viewing public key */
    public masterTaggingPublicKey: PublicKey,
  ) {}

  hash() {
    return poseidon2Hash([
      this.masterNullifierPublicKey,
      this.masterIncomingViewingPublicKey,
      this.masterOutgoingViewingPublicKey,
      this.masterTaggingPublicKey,
      GeneratorIndex.PUBLIC_KEYS_HASH,
    ]);
  }

  /**
   * Determines if this PublicKeys instance is equal to the given PublicKeys instance.
   * Equality is based on the content of their respective buffers.
   *
   * @param other - The PublicKeys instance to compare against.
   * @returns True if the buffers of both instances are equal, false otherwise.
   */
  equals(other: PublicKeys): boolean {
    return (
      this.masterNullifierPublicKey.equals(other.masterNullifierPublicKey) &&
      this.masterIncomingViewingPublicKey.equals(other.masterIncomingViewingPublicKey) &&
      this.masterOutgoingViewingPublicKey.equals(other.masterOutgoingViewingPublicKey) &&
      this.masterTaggingPublicKey.equals(other.masterTaggingPublicKey)
    );
  }

  /**
   * Converts the PublicKeys instance into a Buffer.
   * This method should be used when encoding the address for storage, transmission or serialization purposes.
   *
   * @returns A Buffer representation of the PublicKeys instance.
   */
  toBuffer(): Buffer {
    return serializeToBuffer([
      this.masterNullifierPublicKey,
      this.masterIncomingViewingPublicKey,
      this.masterOutgoingViewingPublicKey,
      this.masterTaggingPublicKey,
    ]);
  }

  /**
   * Creates an PublicKeys instance from a given buffer or BufferReader.
   * If the input is a Buffer, it wraps it in a BufferReader before processing.
   * Throws an error if the input length is not equal to the expected size.
   *
   * @param buffer - The input buffer or BufferReader containing the address data.
   * @returns - A new PublicKeys instance with the extracted address data.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PublicKeys {
    const reader = BufferReader.asReader(buffer);
    const masterNullifierPublicKey = reader.readObject(Point);
    const masterIncomingViewingPublicKey = reader.readObject(Point);
    const masterOutgoingViewingPublicKey = reader.readObject(Point);
    const masterTaggingPublicKey = reader.readObject(Point);
    return new PublicKeys(
      masterNullifierPublicKey,
      masterIncomingViewingPublicKey,
      masterOutgoingViewingPublicKey,
      masterTaggingPublicKey,
    );
  }

  toNoirStruct() {
    // We need to use lowercase identifiers as those are what the noir interface expects
    // eslint-disable-next-line camelcase
    return {
      // TODO(#6337): Directly dump account.publicKeys here
      /* eslint-disable camelcase */
      npk_m: { x: this.masterNullifierPublicKey.x, y: this.masterNullifierPublicKey.y },
      ivpk_m: { x: this.masterIncomingViewingPublicKey.x, y: this.masterIncomingViewingPublicKey.y },
      ovpk_m: { x: this.masterOutgoingViewingPublicKey.x, y: this.masterOutgoingViewingPublicKey.y },
      tpk_m: { x: this.masterTaggingPublicKey.x, y: this.masterTaggingPublicKey.y },
      /* eslint-enable camelcase */
    };
  }

  /**
   * Serializes the payload to an array of fields
   * @returns The fields of the payload
   */
  toFields(): Fr[] {
    return [
      ...this.masterNullifierPublicKey.toFields(),
      ...this.masterIncomingViewingPublicKey.toFields(),
      ...this.masterOutgoingViewingPublicKey.toFields(),
      ...this.masterTaggingPublicKey.toFields(),
    ];
  }
}

import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { computePartialAddress } from '../contract/contract_address.js';
import { computeAddress, deriveKeys } from '../keys/index.js';
import { type PartialAddress } from '../types/partial_address.js';
import { PublicKeys } from '../types/public_keys.js';

/**
 * A complete address is a combination of an Aztec address, a public key and a partial address.
 *
 * @remarks We have introduced this type because it is common that these 3 values are used together. They are commonly
 *          used together because it is the information needed to send user a note.
 * @remarks See the link below for details about how address is computed:
 *          https://github.com/AztecProtocol/aztec-packages/blob/master/docs/docs/concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys
 */
export class CompleteAddress {
  public constructor(
    /** Contract address (typically of an account contract) */
    public address: AztecAddress,
    /** User public keys */
    public publicKeys: PublicKeys,
    /** Partial key corresponding to the public key to the address. */
    public partialAddress: PartialAddress,
  ) {
    this.validate();
  }

  /** Size in bytes of an instance */
  static readonly SIZE_IN_BYTES = 32 * 4;

  static random(): CompleteAddress {
    return this.fromSecretKeyAndPartialAddress(Fr.random(), Fr.random());
  }

  static fromSecretKeyAndPartialAddress(secretKey: Fr, partialAddress: Fr): CompleteAddress {
    const { publicKeys } = deriveKeys(secretKey);
    const address = computeAddress(publicKeys.hash(), partialAddress);
    return new CompleteAddress(address, publicKeys, partialAddress);
  }

  static fromSecretKeyAndInstance(
    secretKey: Fr,
    instance: Parameters<typeof computePartialAddress>[0],
  ): CompleteAddress {
    const partialAddress = computePartialAddress(instance);
    return CompleteAddress.fromSecretKeyAndPartialAddress(secretKey, partialAddress);
  }

  /** Throws if the address is not correctly derived from the public key and partial address.*/
  public validate() {
    const expectedAddress = computeAddress(this.publicKeys.hash(), this.partialAddress);
    if (!expectedAddress.equals(this.address)) {
      throw new Error(
        `Address cannot be derived from public keys and partial address (received ${this.address.toString()}, derived ${expectedAddress.toString()})`,
      );
    }
  }

  /**
   * Gets a readable string representation of the complete address.
   * @returns A readable string representation of the complete address.
   */
  public toReadableString(): string {
    return `Address: ${this.address.toString()}\nMaster Nullifier Public Key: ${this.publicKeys.masterNullifierPublicKey.toString()}\nMaster Incoming Viewing Public Key: ${this.publicKeys.masterIncomingViewingPublicKey.toString()}\nMaster Outgoing Viewing Public Key: ${this.publicKeys.masterOutgoingViewingPublicKey.toString()}\nMaster Tagging Public Key: ${this.publicKeys.masterTaggingPublicKey.toString()}\nPartial Address: ${this.partialAddress.toString()}\n`;
  }

  /**
   * Determines if this CompleteAddress instance is equal to the given CompleteAddress instance.
   * Equality is based on the content of their respective buffers.
   *
   * @param other - The CompleteAddress instance to compare against.
   * @returns True if the buffers of both instances are equal, false otherwise.
   */
  equals(other: CompleteAddress): boolean {
    return (
      this.address.equals(other.address) &&
      this.publicKeys.equals(other.publicKeys) &&
      this.partialAddress.equals(other.partialAddress)
    );
  }

  /**
   * Converts the CompleteAddress instance into a Buffer.
   * This method should be used when encoding the address for storage, transmission or serialization purposes.
   *
   * @returns A Buffer representation of the CompleteAddress instance.
   */
  toBuffer(): Buffer {
    return serializeToBuffer([this.address, this.publicKeys, this.partialAddress]);
  }

  /**
   * Creates an CompleteAddress instance from a given buffer or BufferReader.
   * If the input is a Buffer, it wraps it in a BufferReader before processing.
   * Throws an error if the input length is not equal to the expected size.
   *
   * @param buffer - The input buffer or BufferReader containing the address data.
   * @returns - A new CompleteAddress instance with the extracted address data.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CompleteAddress {
    const reader = BufferReader.asReader(buffer);
    const address = reader.readObject(AztecAddress);
    const publicKeys = reader.readObject(PublicKeys);
    const partialAddress = reader.readObject(Fr);
    return new CompleteAddress(address, publicKeys, partialAddress);
  }

  /**
   * Create a CompleteAddress instance from a hex-encoded string.
   * The input 'address' should be prefixed with '0x' or not, and have exactly 128 hex characters representing the x and y coordinates.
   * Throws an error if the input length is invalid or coordinate values are out of range.
   *
   * @param address - The hex-encoded string representing the complete address.
   * @returns A Point instance.
   */
  static fromString(address: string): CompleteAddress {
    return this.fromBuffer(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  /**
   * Convert the CompleteAddress to a hexadecimal string representation, with a "0x" prefix.
   * The resulting string will have a length of 66 characters (including the prefix).
   *
   * @returns A hexadecimal string representation of the CompleteAddress.
   */
  toString(): string {
    return `0x${this.toBuffer().toString('hex')}`;
  }
}

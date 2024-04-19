import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { computeContractAddressFromPartial, computePartialAddress } from '../contract/contract_address.js';
import { deriveKeys } from '../keys/index.js';
import { type PartialAddress } from '../types/partial_address.js';
import { type PublicKey } from '../types/public_key.js';

/**
 * A complete address is a combination of an Aztec address, a public key and a partial address.
 *
 * @remarks We have introduced this type because it is common that these 3 values are used together. They are commonly
 *          used together because it is the information needed to send user a note.
 * @remarks See the link below for details about how address is computed:
 *          https://github.com/AztecProtocol/aztec-packages/blob/master/docs/docs/concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys
 */
export class CompleteAddress {
  // TODO: This constructor should be private so that the check in create method is always enforced. However, this is
  //       not possible now because we need the class to be compatible with `StringIOClass` to be able to pass it
  //       through `JsonRpcServer`.
  public constructor(
    /** Contract address (typically of an account contract) */
    public address: AztecAddress,
    /** Public key corresponding to the address (used during note encryption). */
    public publicKey: PublicKey,
    /** Partial key corresponding to the public key to the address. */
    public partialAddress: PartialAddress,
  ) {}

  /** Size in bytes of an instance */
  static readonly SIZE_IN_BYTES = 32 * 4;

  static create(address: AztecAddress, publicKey: PublicKey, partialAddress: PartialAddress) {
    const completeAddress = new CompleteAddress(address, publicKey, partialAddress);
    // TODO(#5834): re-enable validation
    // completeAddress.validate();
    return completeAddress;
  }

  static random() {
    // TODO(#5834): the following should be cleaned up
    const secretKey = Fr.random();
    const partialAddress = Fr.random();
    const address = computeContractAddressFromPartial({ secretKey, partialAddress });
    const publicKey = deriveKeys(secretKey).masterIncomingViewingPublicKey;
    return new CompleteAddress(address, publicKey, partialAddress);
  }

  static fromRandomSecretKey() {
    const secretKey = Fr.random();
    const partialAddress = Fr.random();
    return { secretKey, completeAddress: CompleteAddress.fromSecretKeyAndPartialAddress(secretKey, partialAddress) };
  }

  static fromSecretKeyAndPartialAddress(secretKey: Fr, partialAddress: Fr): CompleteAddress {
    const address = computeContractAddressFromPartial({ secretKey, partialAddress });
    const publicKey = deriveKeys(secretKey).masterIncomingViewingPublicKey;
    return new CompleteAddress(address, publicKey, partialAddress);
  }

  static fromSecretKeyAndInstance(
    secretKey: Fr,
    instance: Parameters<typeof computePartialAddress>[0],
  ): CompleteAddress {
    const partialAddress = computePartialAddress(instance);
    const address = computeContractAddressFromPartial({ secretKey, partialAddress });
    const publicKey = deriveKeys(secretKey).masterIncomingViewingPublicKey;
    return new CompleteAddress(address, publicKey, partialAddress);
  }

  // TODO(#5834): re-enable validation
  // /** Throws if the address is not correctly derived from the public key and partial address.*/
  // public validate() {
  //   const expectedAddress = computeContractAddressFromPartial(this);
  //   const address = this.address;
  //   if (!expectedAddress.equals(address)) {
  //     throw new Error(
  //       `Address cannot be derived from pubkey and partial address (received ${address.toString()}, derived ${expectedAddress.toString()})`,
  //     );
  //   }
  // }

  /**
   * Gets a readable string representation of a the complete address.
   * @returns A readable string representation of the complete address.
   */
  public toReadableString(): string {
    return ` Address: ${this.address.toString()}\n Public Key: ${this.publicKey.toString()}\n Partial Address: ${this.partialAddress.toString()}\n`;
  }

  /**
   * Determines if this CompleteAddress instance is equal to the given CompleteAddress instance.
   * Equality is based on the content of their respective buffers.
   *
   * @param other - The CompleteAddress instance to compare against.
   * @returns True if the buffers of both instances are equal, false otherwise.
   */
  equals(other: CompleteAddress) {
    return (
      this.address.equals(other.address) &&
      this.publicKey.equals(other.publicKey) &&
      this.partialAddress.equals(other.partialAddress)
    );
  }

  /**
   * Converts the CompleteAddress instance into a Buffer.
   * This method should be used when encoding the address for storage, transmission or serialization purposes.
   *
   * @returns A Buffer representation of the CompleteAddress instance.
   */
  toBuffer() {
    return Buffer.concat([this.address.toBuffer(), this.publicKey.toBuffer(), this.partialAddress.toBuffer()]);
  }

  /**
   * Creates an CompleteAddress instance from a given buffer or BufferReader.
   * If the input is a Buffer, it wraps it in a BufferReader before processing.
   * Throws an error if the input length is not equal to the expected size.
   *
   * @param buffer - The input buffer or BufferReader containing the address data.
   * @returns - A new CompleteAddress instance with the extracted address data.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    const address = reader.readObject(AztecAddress);
    const publicKey = reader.readObject(Point);
    const partialAddress = reader.readObject(Fr);
    return new this(address, publicKey, partialAddress);
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

import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, type Tuple } from '@aztec/foundation/serialize';

import { computePartialAddress } from '../contract/contract_address.js';
import { computeAddress, computePublicKeysHash, deriveKeys } from '../keys/index.js';
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
    /** Master nullifier public key */
    public masterNullifierPublicKey: PublicKey,
    /** Master incoming viewing public key */
    public masterIncomingViewingPublicKey: PublicKey,
    /** Master outgoing viewing public key */
    public masterOutgoingViewingPublicKey: PublicKey,
    /** Master tagging viewing public key */
    public masterTaggingPublicKey: PublicKey,
    /** Partial key corresponding to the public key to the address. */
    public partialAddress: PartialAddress,
  ) {}

  /** Size in bytes of an instance */
  static readonly SIZE_IN_BYTES = 32 * 4;

  static create(
    address: AztecAddress,
    masterNullifierPublicKey: PublicKey,
    masterIncomingViewingPublicKey: PublicKey,
    masterOutgoingViewingPublicKey: PublicKey,
    masterTaggingPublicKey: PublicKey,
    partialAddress: PartialAddress,
  ): CompleteAddress {
    const completeAddress = new CompleteAddress(
      address,
      masterNullifierPublicKey,
      masterIncomingViewingPublicKey,
      masterOutgoingViewingPublicKey,
      masterTaggingPublicKey,
      partialAddress,
    );
    completeAddress.validate();
    return completeAddress;
  }

  static random(): CompleteAddress {
    return this.fromSecretKeyAndPartialAddress(Fr.random(), Fr.random());
  }

  static fromSecretKeyAndPartialAddress(secretKey: Fr, partialAddress: Fr): CompleteAddress {
    const {
      masterNullifierPublicKey,
      masterIncomingViewingPublicKey,
      masterOutgoingViewingPublicKey,
      masterTaggingPublicKey,
      publicKeysHash,
    } = deriveKeys(secretKey);
    const address = computeAddress(publicKeysHash, partialAddress);
    return new CompleteAddress(
      address,
      masterNullifierPublicKey,
      masterIncomingViewingPublicKey,
      masterOutgoingViewingPublicKey,
      masterTaggingPublicKey,
      partialAddress,
    );
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
    const publicKeysHash = computePublicKeysHash(
      this.masterNullifierPublicKey,
      this.masterIncomingViewingPublicKey,
      this.masterOutgoingViewingPublicKey,
      this.masterTaggingPublicKey,
    );
    const expectedAddress = computeAddress(publicKeysHash, this.partialAddress);
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
    return `Address: ${this.address.toString()}\nMaster Nullifier Public Key: ${this.masterNullifierPublicKey.toString()}\nMaster Incoming Viewing Public Key: ${this.masterIncomingViewingPublicKey.toString()}\nMaster Outgoing Viewing Public Key: ${this.masterOutgoingViewingPublicKey.toString()}\nMaster Tagging Public Key: ${this.masterTaggingPublicKey.toString()}\nPartial Address: ${this.partialAddress.toString()}\n`;
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
      this.masterNullifierPublicKey.equals(other.masterNullifierPublicKey) &&
      this.masterIncomingViewingPublicKey.equals(other.masterIncomingViewingPublicKey) &&
      this.masterOutgoingViewingPublicKey.equals(other.masterOutgoingViewingPublicKey) &&
      this.masterTaggingPublicKey.equals(other.masterTaggingPublicKey) &&
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
    return Buffer.concat([
      this.address.toBuffer(),
      this.masterNullifierPublicKey.toBuffer(),
      this.masterIncomingViewingPublicKey.toBuffer(),
      this.masterOutgoingViewingPublicKey.toBuffer(),
      this.masterTaggingPublicKey.toBuffer(),
      this.partialAddress.toBuffer(),
    ]);
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
    const masterNullifierPublicKey = reader.readObject(Point);
    const masterIncomingViewingPublicKey = reader.readObject(Point);
    const masterOutgoingViewingPublicKey = reader.readObject(Point);
    const masterTaggingPublicKey = reader.readObject(Point);
    const partialAddress = reader.readObject(Fr);
    return new CompleteAddress(
      address,
      masterNullifierPublicKey,
      masterIncomingViewingPublicKey,
      masterOutgoingViewingPublicKey,
      masterTaggingPublicKey,
      partialAddress,
    );
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

  get publicKeys(): Tuple<PublicKey, 4> {
    return [
      this.masterNullifierPublicKey,
      this.masterIncomingViewingPublicKey,
      this.masterOutgoingViewingPublicKey,
      this.masterTaggingPublicKey,
    ];
  }

  get publicKeysHash(): Fr {
    return computePublicKeysHash(...this.publicKeys);
  }
}

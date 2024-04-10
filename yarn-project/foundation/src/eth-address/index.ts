import { inspect } from 'util';

import { keccak256String } from '../crypto/keccak/index.js';
import { randomBytes } from '../crypto/random/index.js';
import { Fr } from '../fields/index.js';
import { BufferReader, FieldReader } from '../serialize/index.js';
import { TypeRegistry } from '../serialize/type_registry.js';

/**
 * Represents an Ethereum address as a 20-byte buffer and provides various utility methods
 * for converting between different representations, generating random addresses, validating
 * checksums, and comparing addresses. EthAddress can be instantiated using a buffer or string,
 * and can be serialized/deserialized from a buffer or BufferReader.
 */
export class EthAddress {
  /**
   * The size of an Ethereum address in bytes.
   */
  public static SIZE_IN_BYTES = 20;
  /**
   * Represents a zero Ethereum address with 20 bytes filled with zeros.
   */
  public static ZERO = new EthAddress(Buffer.alloc(EthAddress.SIZE_IN_BYTES));

  constructor(private buffer: Buffer) {
    if (buffer.length !== EthAddress.SIZE_IN_BYTES) {
      throw new Error(`Expect buffer size to be ${EthAddress.SIZE_IN_BYTES}. Got ${buffer.length}.`);
    }
  }

  /**
   * Creates an EthAddress instance from a valid Ethereum address string.
   * The input 'address' can be either in checksum format or lowercase, and it can be prefixed with '0x'.
   * Throws an error if the input is not a valid Ethereum address.
   *
   * @param address - The string representing the Ethereum address.
   * @returns An EthAddress instance.
   */
  public static fromString(address: string) {
    if (!EthAddress.isAddress(address)) {
      throw new Error(`Invalid address string: ${address}`);
    }
    return new EthAddress(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  /**
   * Create a random EthAddress instance with 20 random bytes.
   * This method generates a new Ethereum address with a randomly generated set of 20 bytes.
   * It is useful for generating test addresses or unique identifiers.
   *
   * @returns A randomly generated EthAddress instance.
   */
  public static random() {
    return new EthAddress(randomBytes(20));
  }

  /**
   * Determines if the given string represents a valid Ethereum address.
   * A valid address should meet the following criteria:
   * 1. Contains exactly 40 hex characters (excluding an optional '0x' prefix).
   * 2. Is either all lowercase, all uppercase, or has a valid checksum based on EIP-55.
   *
   * @param address - The string to be checked for validity as an Ethereum address.
   * @returns True if the input string represents a valid Ethereum address, false otherwise.
   */
  public static isAddress(address: string) {
    if (!/^(0x)?[0-9a-f]{40}$/i.test(address)) {
      // Does not have the basic requirements of an address.
      return false;
    } else if (/^(0x|0X)?[0-9a-f]{40}$/.test(address) || /^(0x|0X)?[0-9A-F]{40}$/.test(address)) {
      // It's ALL lowercase or ALL uppercase.
      return true;
    } else {
      return EthAddress.checkAddressChecksum(address);
    }
  }

  /**
   * Checks if the EthAddress instance represents a zero address.
   * A zero address consists of 20 bytes filled with zeros and is considered an invalid address.
   *
   * @returns A boolean indicating whether the EthAddress instance is a zero address or not.
   */
  public isZero() {
    return this.equals(EthAddress.ZERO);
  }

  /**
   * Checks if the given Ethereum address has a valid checksum.
   * The input 'address' should be prefixed with '0x' or not, and have exactly 40 hex characters.
   * Returns true if the address has a valid checksum, false otherwise.
   *
   * @param address - The hex-encoded string representing the Ethereum address.
   * @returns A boolean value indicating whether the address has a valid checksum.
   */
  public static checkAddressChecksum(address: string) {
    address = address.replace(/^0x/i, '');
    const addressHash = keccak256String(address.toLowerCase());

    for (let i = 0; i < 40; i++) {
      // The nth letter should be uppercase if the nth digit of casemap is 1.
      if (
        (parseInt(addressHash[i], 16) > 7 && address[i].toUpperCase() !== address[i]) ||
        (parseInt(addressHash[i], 16) <= 7 && address[i].toLowerCase() !== address[i])
      ) {
        return false;
      }
    }
    return true;
  }

  /**
   * Converts an Ethereum address to its checksum format.
   * The input 'address' should be prefixed with '0x' or not, and have exactly 40 hex characters.
   * The checksum format is created by capitalizing certain characters in the hex string
   * based on the hash of the lowercase address.
   * Throws an error if the input address is invalid.
   *
   * @param address - The Ethereum address as a hex-encoded string.
   * @returns The Ethereum address in its checksum format.
   */
  public static toChecksumAddress(address: string) {
    if (!EthAddress.isAddress(address)) {
      throw new Error('Invalid address string.');
    }

    address = address.toLowerCase().replace(/^0x/i, '');
    const addressHash = keccak256String(address);
    let checksumAddress = '0x';

    for (let i = 0; i < address.length; i++) {
      // If ith character is 9 to f then make it uppercase.
      if (parseInt(addressHash[i], 16) > 7) {
        checksumAddress += address[i].toUpperCase();
      } else {
        checksumAddress += address[i];
      }
    }
    return checksumAddress;
  }

  /**
   * Checks whether the given EthAddress instance is equal to the current instance.
   * Equality is determined by comparing the underlying byte buffers of both instances.
   *
   * @param rhs - The EthAddress instance to compare with the current instance.
   * @returns A boolean value indicating whether the two instances are equal (true) or not (false).
   */
  public equals(rhs: EthAddress) {
    return this.buffer.equals(rhs.buffer);
  }

  /**
   * Converts the Ethereum address to a hex-encoded string.
   * The resulting string is prefixed with '0x' and has exactly 40 hex characters.
   * This method can be used to represent the EthAddress instance in the widely used hexadecimal format.
   *
   * @returns A hex-encoded string representation of the Ethereum address.
   */
  public toString() {
    return `0x${this.buffer.toString('hex')}` as `0x${string}`;
  }

  [inspect.custom]() {
    return `EthAddress<${this.toString()}>`;
  }

  /**
   * Returns the Ethereum address as a checksummed string.
   * The output string will have characters in the correct upper or lowercase form, according to EIP-55.
   * This provides a way to verify if an address is typed correctly, by checking the character casing.
   *
   * @returns A checksummed Ethereum address string.
   */
  public toChecksumString() {
    return EthAddress.toChecksumAddress(this.buffer.toString('hex'));
  }

  /**
   * Returns a 20-byte buffer representation of the Ethereum address.
   * @returns A 20-byte Buffer containing the Ethereum address.
   */
  public toBuffer() {
    return this.buffer;
  }

  /**
   * Returns a 32-byte buffer representation of the Ethereum address, with the original 20-byte address
   * occupying the last 20 bytes and the first 12 bytes being zero-filled.
   * This format is commonly used in smart contracts when handling addresses as 32-byte values.
   *
   * @returns A 32-byte Buffer containing the padded Ethereum address.
   */
  public toBuffer32() {
    const buffer = Buffer.alloc(32);
    this.buffer.copy(buffer, 12);
    return buffer;
  }

  /**
   * Returns a new field with the same contents as this EthAddress.
   *
   * @returns An Fr instance.
   */
  public toField() {
    return Fr.fromBuffer(this.toBuffer32());
  }

  /**
   * Converts a field to a eth address.
   * @param fr - The field to convert.
   * @returns The eth address.
   */
  static fromField(fr: Fr): EthAddress {
    return new EthAddress(fr.toBuffer().slice(-EthAddress.SIZE_IN_BYTES));
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return EthAddress.fromField(reader.readField());
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   * @returns The EthAddress.
   */
  static fromBuffer(buffer: Buffer | BufferReader): EthAddress {
    const reader = BufferReader.asReader(buffer);
    return new EthAddress(reader.readBytes(EthAddress.SIZE_IN_BYTES));
  }

  /**
   * Friendly representation for debugging purposes.
   * @returns A hex string representing the address.
   */
  toFriendlyJSON() {
    return this.toString();
  }

  toJSON() {
    return {
      type: 'EthAddress',
      value: this.toString(),
    };
  }
}

// For deserializing JSON.
TypeRegistry.register('EthAddress', EthAddress);

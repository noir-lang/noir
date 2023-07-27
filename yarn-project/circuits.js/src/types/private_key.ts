import { randomBytes } from '@aztec/foundation/crypto';

/**
 * A wrapper around a buffer representation of a private key with validation checks.
 */
export class PrivateKey {
  constructor(
    /** A buffer containing the private key. */
    public value: Buffer,
  ) {
    if (value.length != 32) {
      throw new Error(`Invalid private key length. Got ${value.length}, expected 32.`);
    }
  }

  /**
   * Converts the private key to a hex string.
   * @returns A hex string representation of the private key.
   */
  public toString(): string {
    return this.value.toString('hex');
  }

  /**
   * Creates a PrivateKey instance from a hex string.
   * @param keyString - A hex string representation of the private key.
   * @returns A PrivateKey instance.
   */
  public static fromString(keyString: string): PrivateKey {
    if (keyString.startsWith('0x')) {
      keyString = keyString.slice(2);
    }
    if (keyString.length != 64) {
      throw new Error(`Invalid private key string length. Got ${keyString.length}, expected 64.`);
    }
    return new PrivateKey(Buffer.from(keyString, 'hex'));
  }

  /**
   * Creates a random PrivateKey instance.
   * @returns A PrivateKey instance.
   */
  public static random(): PrivateKey {
    return new PrivateKey(randomBytes(32));
  }
}

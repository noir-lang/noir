import { STRING_ENCODING } from '@aztec/circuits.js';
import { serializeToBuffer } from '@aztec/circuits.js/utils';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

/**
 * Write operations on the public state tree.
 */
export class PublicDataWrite {
  constructor(
    /**
     * Index of the updated leaf.
     */
    public readonly leafIndex: Fr,
    /**
     * New value of the leaf.
     */
    public readonly newValue: Fr,
  ) {}

  /**
   * Creates a new public data write operation from the given arguments.
   * @param args - Arguments containing info used to create a new public data write operation.
   * @returns A new public data write operation instance.
   */
  static from(args: {
    /**
     * Index of the updated leaf.
     */
    leafIndex: Fr;
    /**
     * New value of the leaf.
     */
    newValue: Fr;
  }) {
    return new PublicDataWrite(args.leafIndex, args.newValue);
  }

  /**
   * Serializes the public data write operation to a buffer.
   * @returns A buffer containing the serialized public data write operation.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.leafIndex, this.newValue);
  }

  /**
   * Serialises the operation to a string.
   * @returns A string representation of the operation.
   */
  toString(): string {
    return this.toBuffer().toString(STRING_ENCODING);
  }

  /**
   * Checks if the public data write operation is empty.
   * @returns True if the public data write operation is empty, false otherwise.
   */
  isEmpty(): boolean {
    return this.leafIndex.isZero() && this.newValue.isZero();
  }

  /**
   * Creates a new public data write operation from the given buffer.
   * @param buffer - Buffer containing the serialized public data write operation.
   * @returns A new public data write operation instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PublicDataWrite {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataWrite(reader.readFr(), reader.readFr());
  }

  /**
   * Creates a new public data write operation from the given string.
   * @param str - The serialised string
   * @returns A new public data write operation instance.
   */
  static fromString(str: string): PublicDataWrite {
    return PublicDataWrite.fromBuffer(Buffer.from(str, STRING_ENCODING));
  }

  /**
   * Creates an empty public data write operation.
   * @returns A new public data write operation instance.
   */
  static empty(): PublicDataWrite {
    return new PublicDataWrite(Fr.ZERO, Fr.ZERO);
  }

  /**
   * Creates a random public data write operation.
   * @returns A new public data write operation instance.
   */
  static random(): PublicDataWrite {
    return new PublicDataWrite(Fr.random(), Fr.random());
  }
}

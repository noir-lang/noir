import { Vector } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

/**
 * The NotePreimage class represents a vector of Fr (finite field) elements, used for constructing
 * and manipulating preimages of zk-SNARK commitments in the AZTEC protocol. This class provides
 * methods to create a NotePreimage instance from a buffer or generate a random one. It extends
 * the Vector class, which allows for additional operations on the underlying field elements.
 */
export class NotePreimage extends Vector<Fr> {
  /**
   * Create a NotePreimage instance from a Buffer or BufferReader.
   * The input 'buffer' can be either a Buffer containing the serialized Fr elements or a BufferReader instance.
   * This function reads the Fr elements in the buffer and constructs a NotePreimage with them.
   *
   * @param buffer - The Buffer or BufferReader containing the serialized Fr elements.
   * @returns A NotePreimage instance containing the deserialized Fr elements.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NotePreimage(reader.readVector(Fr));
  }

  /**
   * Generates a random NotePreimage instance with a variable number of items.
   * The number of items is determined by a random value between 1 and 10 (inclusive).
   * Each item in the NotePreimage is generated using the Fr.random() method.
   *
   * @returns A randomly generated NotePreimage instance.
   */
  static random() {
    const numItems = Math.floor(Math.random() * 10) + 1;
    const items = Array.from({ length: numItems }, () => Fr.random());
    return new NotePreimage(items);
  }

  /**
   * Returns a hex representation of this preimage.
   * @returns A hex string with the vector length as first element.
   */
  toString() {
    return '0x' + this.toBuffer().toString('hex');
  }

  /**
   * Creates a new NotePreimage instance from a hex string.
   * @param str - Hex representation.
   * @returns A NotePreimage instance.
   */
  static fromString(str: string) {
    const hex = str.replace(/^0x/, '');
    return NotePreimage.fromBuffer(Buffer.from(hex, 'hex'));
  }
}

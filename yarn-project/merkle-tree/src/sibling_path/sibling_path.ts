import { Pedersen } from '../pedersen.js';
import { deserializeArrayFromVector, serializeBufferArrayToVector } from '@aztec/foundation';

/**
 * Contains functionality to compute and serialize/deserialize a sibling path.
 * e.g. Sibling path for a leaf at index 3 in a tree of depth 3 consists of:
 *      d0:                                            [ root ]
 *      d1:                      [ ]                                               [*]
 *      d2:         [*]                      [ ]                       [ ]                     [ ]
 *      d3:   [ ]         [ ]          [*]         [ ]           [ ]         [ ]          [ ]        [ ]
 *
 *      And the elements would be ordered as: [ leaf_at_index_2, node_at_level_2_index_0, node_at_level_1_index_1 ]
 */
export class SiblingPath {
  /**
   * Returns sibling path hashed up from the a element.
   * @param size - The number of elements in a given path.
   * @param zeroElement - Value of the zero element.
   * @param pedersen - Implementation of a hasher interface using the Pedersen hash.
   * @returns A sibling path hashed up from a zero element.
   */
  public static ZERO(size: number, zeroElement: Buffer, pedersen: Pedersen): SiblingPath {
    const bufs: Buffer[] = [];
    let current = zeroElement;
    for (let i = 0; i < size; ++i) {
      bufs.push(current);
      current = pedersen.compress(current, current);
    }
    return new SiblingPath(bufs);
  }

  /**
   * Constructor.
   * @param data - The sibling path data.
   */
  constructor(
    /**
     * The sibling path data.
     */
    public data: Buffer[] = [],
  ) {}

  /**
   * Serializes this SiblingPath object to a buffer.
   * @returns The buffer representation of this object.
   */
  public toBuffer(): Buffer {
    return serializeBufferArrayToVector(this.data);
  }

  /**
   * Deserializes a SiblingPath from a buffer.
   * @param buf - A buffer containing the buffer representation of SiblingPath.
   * @param offset - An offset to start deserializing from.
   * @returns A SiblingPath object.
   */
  static fromBuffer(buf: Buffer, offset = 0): SiblingPath {
    const { elem } = SiblingPath.deserialize(buf, offset);
    return elem;
  }

  /**
   * Deserializes a SiblingPath object from a slice of a part of a buffer and returns the amount of bytes advanced.
   * @param buf - A buffer representation of the sibling path.
   * @param offset - An offset to start deserializing from.
   * @returns The deserialized sibling path and the number of bytes advanced.
   */
  static deserialize(buf: Buffer, offset = 0) {
    const deserializePath = (buf: Buffer, offset: number) => ({
      elem: buf.slice(offset, offset + 32),
      adv: 32,
    });
    const { elem, adv } = deserializeArrayFromVector(deserializePath, buf, offset);
    return { elem: new SiblingPath(elem), adv };
  }

  /**
   * Serializes this SiblingPath object to a hex string representation.
   * @returns A hex string representation of the sibling path.
   */
  public toString(): string {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes a SiblingPath object from a hex string representation.
   * @param repr - A hex string representation of the sibling path.
   * @returns A SiblingPath object.
   */
  public static fromString(repr: string): SiblingPath {
    return SiblingPath.fromBuffer(Buffer.from(repr, 'hex'));
  }
}

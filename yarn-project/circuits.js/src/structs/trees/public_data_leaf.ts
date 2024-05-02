import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type IndexedTreeLeaf, type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

/**
 * Class containing the data of a preimage of a single leaf in the public data tree.
 * Note: It's called preimage because this data gets hashed before being inserted as a node into the `IndexedTree`.
 */
export class PublicDataTreeLeafPreimage implements IndexedTreeLeafPreimage {
  constructor(
    /**
     * The slot of the leaf
     */
    public slot: Fr,
    /**
     * The value of the leaf
     */
    public value: Fr,
    /**
     * Next value inside the indexed tree's linked list.
     */
    public nextSlot: Fr,
    /**
     * Index of the next leaf in the indexed tree's linked list.
     */
    public nextIndex: bigint,
  ) {}

  getKey(): bigint {
    return this.slot.toBigInt();
  }

  getNextKey(): bigint {
    return this.nextSlot.toBigInt();
  }

  getNextIndex(): bigint {
    return this.nextIndex;
  }

  asLeaf(): PublicDataTreeLeaf {
    return new PublicDataTreeLeaf(this.slot, this.value);
  }

  toBuffer(): Buffer {
    return Buffer.concat(this.toHashInputs());
  }

  toHashInputs(): Buffer[] {
    return [
      Buffer.from(this.slot.toBuffer()),
      Buffer.from(this.value.toBuffer()),
      Buffer.from(toBufferBE(this.nextIndex, 32)),
      Buffer.from(this.nextSlot.toBuffer()),
    ];
  }

  clone(): PublicDataTreeLeafPreimage {
    return new PublicDataTreeLeafPreimage(this.slot, this.value, this.nextSlot, this.nextIndex);
  }

  static empty(): PublicDataTreeLeafPreimage {
    return new PublicDataTreeLeafPreimage(Fr.ZERO, Fr.ZERO, Fr.ZERO, 0n);
  }

  static fromBuffer(buffer: Buffer | BufferReader): PublicDataTreeLeafPreimage {
    const reader = BufferReader.asReader(buffer);
    const slot = Fr.fromBuffer(reader);
    const value = Fr.fromBuffer(reader);
    const nextIndex = toBigIntBE(reader.readBytes(32));
    const nextSlot = Fr.fromBuffer(reader);
    return new PublicDataTreeLeafPreimage(slot, value, nextSlot, nextIndex);
  }

  static fromLeaf(leaf: PublicDataTreeLeaf, nextKey: bigint, nextIndex: bigint): PublicDataTreeLeafPreimage {
    return new PublicDataTreeLeafPreimage(leaf.slot, leaf.value, new Fr(nextKey), nextIndex);
  }

  static clone(preimage: PublicDataTreeLeafPreimage): PublicDataTreeLeafPreimage {
    return new PublicDataTreeLeafPreimage(preimage.slot, preimage.value, preimage.nextSlot, preimage.nextIndex);
  }
}

/**
 * A leaf in the public data indexed tree.
 */
export class PublicDataTreeLeaf implements IndexedTreeLeaf {
  constructor(
    /**
     * The slot the value is stored in
     */
    public slot: Fr,
    /**
     * The value stored in the slot
     */
    public value: Fr,
  ) {}

  getKey(): bigint {
    return this.slot.toBigInt();
  }

  toBuffer() {
    return serializeToBuffer([this.slot, this.value]);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataTreeLeaf(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  equals(another: PublicDataTreeLeaf): boolean {
    return this.slot.equals(another.slot) && this.value.equals(another.value);
  }

  toString(): string {
    return `PublicDataTreeLeaf(${this.slot.toString()}, ${this.value.toString()})`;
  }

  isEmpty(): boolean {
    return this.slot.isZero() && this.value.isZero();
  }

  updateTo(another: PublicDataTreeLeaf): PublicDataTreeLeaf {
    if (!this.slot.equals(another.slot)) {
      throw new Error('Invalid update: slots do not match');
    }
    return new PublicDataTreeLeaf(this.slot, another.value);
  }

  static buildDummy(key: bigint): PublicDataTreeLeaf {
    return new PublicDataTreeLeaf(new Fr(key), new Fr(0));
  }

  static empty(): PublicDataTreeLeaf {
    return new PublicDataTreeLeaf(Fr.ZERO, Fr.ZERO);
  }
}

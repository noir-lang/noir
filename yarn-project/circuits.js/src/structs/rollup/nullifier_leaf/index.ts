import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';
import { type IndexedTreeLeaf, type IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

/**
 * Class containing the data of a preimage of a single leaf in the nullifier tree.
 * Note: It's called preimage because this data gets hashed before being inserted as a node into the `IndexedTree`.
 */
export class NullifierLeafPreimage implements IndexedTreeLeafPreimage {
  constructor(
    /**
     * Leaf value inside the indexed tree's linked list.
     */
    public nullifier: Fr,
    /**
     * Next value inside the indexed tree's linked list.
     */
    public nextNullifier: Fr,
    /**
     * Index of the next leaf in the indexed tree's linked list.
     */
    public nextIndex: bigint,
  ) {}

  getKey(): bigint {
    return this.nullifier.toBigInt();
  }

  getNextKey(): bigint {
    return this.nextNullifier.toBigInt();
  }

  getNextIndex(): bigint {
    return this.nextIndex;
  }

  asLeaf(): NullifierLeaf {
    return new NullifierLeaf(this.nullifier);
  }

  toBuffer(): Buffer {
    return Buffer.concat(this.toHashInputs());
  }

  toHashInputs(): Buffer[] {
    return [
      Buffer.from(this.nullifier.toBuffer()),
      Buffer.from(this.nextNullifier.toBuffer()),
      Buffer.from(toBufferBE(this.nextIndex, 32)),
    ];
  }

  toFields(): Fr[] {
    return this.toHashInputs().map(buf => Fr.fromBuffer(buf));
  }

  clone(): NullifierLeafPreimage {
    return new NullifierLeafPreimage(this.nullifier, this.nextNullifier, this.nextIndex);
  }

  toJSON() {
    return {
      nullifier: this.nullifier.toString(),
      nextNullifier: this.nextNullifier.toString(),
      nextIndex: '0x' + this.nextIndex.toString(16),
    };
  }

  static empty(): NullifierLeafPreimage {
    return new NullifierLeafPreimage(Fr.ZERO, Fr.ZERO, 0n);
  }

  static fromBuffer(buffer: Buffer | BufferReader): NullifierLeafPreimage {
    const reader = BufferReader.asReader(buffer);
    return new NullifierLeafPreimage(reader.readObject(Fr), reader.readObject(Fr), toBigIntBE(reader.readBytes(32)));
  }

  static fromLeaf(leaf: NullifierLeaf, nextKey: bigint, nextIndex: bigint): NullifierLeafPreimage {
    return new NullifierLeafPreimage(leaf.nullifier, new Fr(nextKey), nextIndex);
  }

  static clone(preimage: NullifierLeafPreimage): NullifierLeafPreimage {
    return new NullifierLeafPreimage(preimage.nullifier, preimage.nextNullifier, preimage.nextIndex);
  }

  static fromJSON(json: any): NullifierLeafPreimage {
    return new NullifierLeafPreimage(
      Fr.fromString(json.nullifier),
      Fr.fromString(json.nextNullifier),
      BigInt(json.nextIndex),
    );
  }
}

/**
 * A nullifier to be inserted in the nullifier tree.
 */
export class NullifierLeaf implements IndexedTreeLeaf {
  constructor(
    /**
     * Nullifier value.
     */
    public nullifier: Fr,
  ) {}

  getKey(): bigint {
    return this.nullifier.toBigInt();
  }

  toBuffer(): Buffer {
    return this.nullifier.toBuffer();
  }

  isEmpty(): boolean {
    return this.nullifier.isZero();
  }

  updateTo(_another: NullifierLeaf): NullifierLeaf {
    throw new Error('Nullifiers are create only');
  }

  static buildDummy(key: bigint): NullifierLeaf {
    return new NullifierLeaf(new Fr(key));
  }

  static fromBuffer(buf: Buffer): NullifierLeaf {
    return new NullifierLeaf(Fr.fromBuffer(buf));
  }
}

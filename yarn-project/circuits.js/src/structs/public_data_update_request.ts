import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

/**
 * Write operations on the public data tree including the previous value.
 */
export class PublicDataUpdateRequest {
  constructor(
    /**
     * Index of the leaf in the public data tree which is to be updated.
     */
    public readonly leafSlot: Fr,
    /**
     * New value of the leaf.
     */
    public readonly newValue: Fr,
    /**
     * Optional side effect counter tracking position of this event in tx execution.
     */
    public readonly sideEffectCounter?: number,
  ) {}

  static from(args: {
    /**
     * Index of the leaf in the public data tree which is to be updated.
     */
    leafIndex: Fr;
    /**
     * New value of the leaf.
     */
    newValue: Fr;
  }) {
    return new PublicDataUpdateRequest(args.leafIndex, args.newValue);
  }

  toBuffer() {
    return serializeToBuffer(this.leafSlot, this.newValue);
  }

  isEmpty() {
    return this.leafSlot.isZero() && this.newValue.isZero();
  }

  static isEmpty(x: PublicDataUpdateRequest) {
    return x.isEmpty();
  }

  equals(other: PublicDataUpdateRequest) {
    return this.leafSlot.equals(other.leafSlot) && this.newValue.equals(other.newValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataUpdateRequest(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  static empty() {
    return new PublicDataUpdateRequest(Fr.ZERO, Fr.ZERO);
  }

  toFriendlyJSON() {
    return `Leaf=${this.leafSlot.toFriendlyJSON()}: ${this.newValue.toFriendlyJSON()}`;
  }
}

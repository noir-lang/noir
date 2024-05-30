import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

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
     * Side effect counter tracking position of this event in tx execution.
     */
    public readonly sideEffectCounter: number,
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

    /**
     * Side effect counter tracking position of this event in tx execution.
     */
    sideEffectCounter: number;
  }) {
    return new PublicDataUpdateRequest(args.leafIndex, args.newValue, args.sideEffectCounter);
  }

  get counter() {
    return this.sideEffectCounter;
  }

  get position() {
    return this.leafSlot;
  }

  toBuffer() {
    return serializeToBuffer(this.leafSlot, this.newValue, this.sideEffectCounter);
  }

  isEmpty() {
    return this.leafSlot.isZero() && this.newValue.isZero();
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new PublicDataUpdateRequest(reader.readField(), reader.readField(), reader.readU32());
  }

  static isEmpty(x: PublicDataUpdateRequest) {
    return x.isEmpty();
  }

  equals(other: PublicDataUpdateRequest) {
    return this.leafSlot.equals(other.leafSlot) && this.newValue.equals(other.newValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataUpdateRequest(Fr.fromBuffer(reader), Fr.fromBuffer(reader), reader.readNumber());
  }

  static empty() {
    return new PublicDataUpdateRequest(Fr.ZERO, Fr.ZERO, 0);
  }

  toFriendlyJSON() {
    return `Leaf=${this.leafSlot.toFriendlyJSON()}: ${this.newValue.toFriendlyJSON()}, SideEffectCounter=${
      this.sideEffectCounter
    }`;
  }

  [inspect.custom]() {
    return `PublicDataUpdateRequest { leafSlot: ${this.leafSlot.toFriendlyJSON()}, newValue: ${this.newValue.toFriendlyJSON()}, sideEffectCounter: ${
      this.sideEffectCounter
    } }`;
  }
}

import { BufferReader, randomBytes } from '@aztec/foundation';
import { Fq } from '@aztec/foundation/fields';
import { assertLength } from '../utils/jsUtils.js';
import { Bufferable, serializeToBuffer } from '../utils/serialize.js';

export class Vector<T extends Bufferable> {
  constructor(public items: T[]) {}

  toBuffer() {
    return serializeToBuffer(this.items.length, this.items);
  }
}

export class UInt8Vector {
  constructor(public buffer: Buffer) {}

  toBuffer() {
    return serializeToBuffer(this.buffer.length, this.buffer);
  }

  static fromBuffer(buffer: Buffer | BufferReader): UInt8Vector {
    const reader = BufferReader.asReader(buffer);
    const size = reader.readNumber();
    const buf = reader.readBytes(size);
    return new UInt8Vector(buf);
  }
}

export type UInt32 = number;

/**
 * Affine element of a group, composed of two elements in Fq.
 * cpp/barretenberg/cpp/src/aztec/ecc/groups/affine_element.hpp
 * cpp/barretenberg/cpp/src/aztec/ecc/curves/bn254/g1.hpp
 */
export class AffineElement {
  public x: Fq;
  public y: Fq;

  constructor(x: Fq | bigint, y: Fq | bigint) {
    this.x = typeof x === 'bigint' ? new Fq(x) : x;
    this.y = typeof y === 'bigint' ? new Fq(y) : y;
  }

  toBuffer() {
    return serializeToBuffer(this.x, this.y);
  }

  static fromBuffer(buffer: Buffer | BufferReader): AffineElement {
    const reader = BufferReader.asReader(buffer);
    return new AffineElement(reader.readFq(), reader.readFq());
  }
}

/**
 * ECDSA signature used for transactions.
 * @see cpp/barretenberg/cpp/src/barretenberg/crypto/ecdsa/ecdsa.hpp
 */
export class EcdsaSignature {
  constructor(public r: Buffer, public s: Buffer) {
    assertLength(this, 'r', 32);
    assertLength(this, 's', 32);
  }

  toBuffer() {
    return serializeToBuffer(this.r, this.s);
  }

  public static random() {
    return new EcdsaSignature(randomBytes(32), randomBytes(32));
  }
}

/**
 * Composer prover type.
 */
export enum ComposerType {
  STANDARD = 0,
  TURBO = 1,
  PLOOKUP = 2,
  STANDARD_HONK = 3,
}

/**
 * Rollup types.
 */
export enum RollupTypes {
  Base = 0,
  Merge = 1,
  Root = 2,
}

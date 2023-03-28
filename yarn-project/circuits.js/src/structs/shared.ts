import { BufferReader } from '@aztec/foundation';
import { Fq, Fr } from '@aztec/foundation/fields';
import { assertLength, checkLength, range } from '../utils/jsUtils.js';
import { Bufferable, numToUInt32BE, serializeToBuffer } from '../utils/serialize.js';

export class MembershipWitness<N extends number> {
  constructor(pathSize: N, public leafIndex: UInt32, public siblingPath: Fr[]) {
    checkLength(this.siblingPath, pathSize, 'MembershipWitness.siblingPath');
  }

  toBuffer() {
    return serializeToBuffer(this.leafIndex, ...this.siblingPath);
  }

  static mock(size: number, start: number) {
    return new MembershipWitness(
      size,
      start,
      range(size, start).map(x => new Fr(numToUInt32BE(x, 32))),
    );
  }
}

export class AggregationObject {
  public publicInputs: Vector<Fr>;
  public proofWitnessIndices: Vector<UInt32>;

  constructor(
    public p0: AffineElement,
    public p1: AffineElement,
    publicInputsData: Fr[],
    proofWitnessIndicesData: UInt32[],
    public hasData = false,
  ) {
    this.publicInputs = new Vector(publicInputsData);
    this.proofWitnessIndices = new Vector(proofWitnessIndicesData);
  }

  toBuffer() {
    return serializeToBuffer(this.p0, this.p1, this.publicInputs, this.proofWitnessIndices, this.hasData);
  }

  static fromBuffer(buffer: Buffer | BufferReader): AggregationObject {
    const reader = BufferReader.asReader(buffer);
    return new AggregationObject(
      reader.readObject(AffineElement),
      reader.readObject(AffineElement),
      reader.readVector(Fr),
      reader.readNumberVector(),
      reader.readBoolean(),
    );
  }
}

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
}

export type UInt32 = number;

/**
 * Affine element of a group, composed of two elements in Fq.
 * cpp/barretenberg/cpp/src/aztec/ecc/groups/affine_element.hpp
 * cpp/barretenberg/cpp/src/aztec/ecc/curves/bn254/g1.hpp
 */
export class AffineElement {
  constructor(public x: Fq, public y: Fq) {}

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
  Rollup = 1,
  Merge = 2,
}

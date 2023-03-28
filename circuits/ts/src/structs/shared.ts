import { BufferReader } from '../utils/buffer_reader.js';
import { assertLength, checkLength, range } from '../utils/jsUtils.js';
import { Bufferable, numToUInt32BE, serializeToBuffer } from '../utils/serialize.js';

abstract class Field {
  public static SIZE_IN_BYTES = 32;

  private buffer: Buffer;

  constructor(input: Buffer | number) {
    if (Buffer.isBuffer(input)) {
      if (input.length != Field.SIZE_IN_BYTES) {
        throw new Error(`Unexpected buffer size ${input.length} (expected ${Field.SIZE_IN_BYTES} bytes)`);
      }
      this.buffer = input;
    } else {
      if (BigInt(input) > this.maxValue()) {
        throw new Error(`Input value ${input} too large (expected ${this.maxValue()})`);
      }
      this.buffer = numToUInt32BE(input, 32);
    }
  }

  abstract maxValue(): bigint;

  toString() {
    return '0x' + this.buffer.toString('hex');
  }

  toBuffer() {
    return this.buffer;
  }
}

export class Fr extends Field {
  /**
   * Maximum represntable value in a field is the curve prime minus one.
   * @returns 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000n
   */
  maxValue() {
    return 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n - 1n;
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }
}

export class Fq extends Field {
  /**
   * Maximum represntable vaue in a field is the curve prime minus one.
   * TODO: Find out actual max value for Fq.
   * @returns 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000n
   */
  maxValue() {
    return 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n - 1n;
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }
}

/**
 * For Ethereum addresses, which must be treated as 32 bytes.
 * @param buffer - The 20 byte ethereum address buffer.
 * @returns The 32 byte padded buffer.
 */
function pad32(buffer: Buffer) {
  // Create a 32-byte Buffer filled with zeros
  const paddedBuffer = Buffer.alloc(32);

  // Calculate the padding length
  const paddingLength = paddedBuffer.length - buffer.length;

  // Copy the original Buffer into the padded Buffer with an offset
  buffer.copy(paddedBuffer, paddingLength);
  return paddedBuffer;
}

export class EthAddress {
  static SIZE_IN_BYTES = 20;

  constructor(public readonly buffer: Buffer) {
    if (buffer.length != EthAddress.SIZE_IN_BYTES) {
      throw new Error(`Unexpected buffer size ${buffer.length} (expected ${EthAddress.SIZE_IN_BYTES} bytes)`);
    }
  }

  toString() {
    return '0x' + this.buffer.toString('hex');
  }

  toBuffer() {
    return pad32(this.buffer);
  }
}

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

// TODO: Define proper type for AztecAddress
export type AztecAddress = Fr;

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

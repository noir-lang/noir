import { BarretenbergSync } from '@aztec/bb.js';

import { inspect } from 'util';

import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { randomBytes } from '../crypto/random/index.js';
import { BufferReader } from '../serialize/buffer_reader.js';
import { TypeRegistry } from '../serialize/type_registry.js';

const ZERO_BUFFER = Buffer.alloc(32);

/* eslint-disable @typescript-eslint/no-unsafe-declaration-merging */

/**
 * Represents a field derived from BaseField.
 */
type DerivedField<T extends BaseField> = {
  new (value: any): T;
  /**
   * All derived fields will specify a MODULUS.
   */
  MODULUS: bigint;
};

/**
 * Base field class.
 * Conversions from Buffer to BigInt and vice-versa are not cheap.
 * We allow construction with either form and lazily convert to other as needed.
 * We only check we are within the field modulus when initializing with bigint.
 */
abstract class BaseField {
  static SIZE_IN_BYTES = 32;
  private asBuffer?: Buffer;
  private asBigInt?: bigint;

  /**
   * Return bigint representation.
   * @deprecated Just to get things compiling. Use toBigInt().
   * */
  get value(): bigint {
    return this.toBigInt();
  }

  /** Returns the size in bytes. */
  get size(): number {
    return BaseField.SIZE_IN_BYTES;
  }

  protected constructor(value: number | bigint | boolean | BaseField | Buffer) {
    if (value instanceof Buffer) {
      if (value.length > BaseField.SIZE_IN_BYTES) {
        throw new Error(`Value length ${value.length} exceeds ${BaseField.SIZE_IN_BYTES}`);
      }
      this.asBuffer =
        value.length === BaseField.SIZE_IN_BYTES
          ? value
          : Buffer.concat([Buffer.alloc(BaseField.SIZE_IN_BYTES - value.length), value]);
    } else if (typeof value === 'bigint' || typeof value === 'number' || typeof value === 'boolean') {
      this.asBigInt = BigInt(value);
      if (this.asBigInt >= this.modulus()) {
        throw new Error(`Value 0x${this.asBigInt.toString(16)} is greater or equal to field modulus.`);
      }
    } else if (value instanceof BaseField) {
      this.asBuffer = value.asBuffer;
      this.asBigInt = value.asBigInt;
    } else {
      throw new Error(`Type '${typeof value}' with value '${value}' passed to BaseField ctor.`);
    }
  }

  protected abstract modulus(): bigint;

  /**
   * We return a copy of the Buffer to ensure this remains immutable.
   */
  toBuffer(): Buffer {
    if (!this.asBuffer) {
      this.asBuffer = toBufferBE(this.asBigInt!, 32);
    }
    return Buffer.from(this.asBuffer);
  }

  toString(): `0x${string}` {
    return `0x${this.toBuffer().toString('hex')}`;
  }

  toBigInt(): bigint {
    if (this.asBigInt === undefined) {
      this.asBigInt = toBigIntBE(this.asBuffer!);
      if (this.asBigInt >= this.modulus()) {
        throw new Error(`Value 0x${this.asBigInt.toString(16)} is greater or equal to field modulus.`);
      }
    }
    return this.asBigInt;
  }

  toBool(): boolean {
    return Boolean(this.toBigInt());
  }

  toNumber(): number {
    const value = this.toBigInt();
    if (value > Number.MAX_SAFE_INTEGER) {
      throw new Error(`Value ${value.toString(16)} greater than than max safe integer`);
    }
    return Number(value);
  }

  toShortString(): string {
    const str = this.toString();
    return `${str.slice(0, 10)}...${str.slice(-4)}`;
  }

  equals(rhs: BaseField): boolean {
    return this.toBuffer().equals(rhs.toBuffer());
  }

  lt(rhs: BaseField): boolean {
    return this.toBigInt() < rhs.toBigInt();
  }

  cmp(rhs: BaseField): -1 | 0 | 1 {
    const lhsBigInt = this.toBigInt();
    const rhsBigInt = rhs.toBigInt();
    return lhsBigInt === rhsBigInt ? 0 : lhsBigInt < rhsBigInt ? -1 : 1;
  }

  isZero(): boolean {
    return this.toBuffer().equals(ZERO_BUFFER);
  }

  isEmpty(): boolean {
    return this.isZero();
  }

  toFriendlyJSON(): string {
    return this.toString();
  }

  toField() {
    return this;
  }
}

/**
 * Constructs a field from a Buffer of BufferReader.
 * It maybe not read the full 32 bytes if the Buffer is shorter, but it will padded in BaseField constructor.
 */
export function fromBuffer<T extends BaseField>(buffer: Buffer | BufferReader, f: DerivedField<T>) {
  const reader = BufferReader.asReader(buffer);
  return new f(reader.readBytes(BaseField.SIZE_IN_BYTES));
}

/**
 * Constructs a field from a Buffer, but reduces it first.
 * This requires a conversion to a bigint first so the initial underlying representation will be a bigint.
 */
function fromBufferReduce<T extends BaseField>(buffer: Buffer, f: DerivedField<T>) {
  return new f(toBigIntBE(buffer) % f.MODULUS);
}

/**
 * To ensure a field is uniformly random, it's important to reduce a 512 bit value.
 * If you reduced a 256 bit number, there would a be a high skew in the lower range of the field.
 */
function random<T extends BaseField>(f: DerivedField<T>): T {
  return fromBufferReduce(randomBytes(64), f);
}

/**
 * Constructs a field from a 0x prefixed hex string.
 */
function fromHexString<T extends BaseField>(buf: string, f: DerivedField<T>) {
  const withoutPrefix = buf.replace(/^0x/i, '');
  const checked = withoutPrefix.match(/^[0-9A-F]+$/i)?.[0];
  if (checked === undefined) {
    throw new Error(`Invalid hex-encoded string: "${buf}"`);
  }

  const buffer = Buffer.from(checked.length % 2 === 1 ? '0' + checked : checked, 'hex');

  return new f(buffer);
}

/**
 * Branding to ensure fields are not interchangeable types.
 */
export interface Fr {
  /** Brand. */
  _branding: 'Fr';
}

/**
 * Fr field class.
 * @dev This class is used to represent elements of BN254 scalar field or elements in the base field of Grumpkin.
 * (Grumpkin's scalar field corresponds to BN254's base field and vice versa.)
 */
export class Fr extends BaseField {
  static ZERO = new Fr(0n);
  static ONE = new Fr(1n);
  static MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n;
  static MAX_FIELD_VALUE = new Fr(this.MODULUS - 1n);

  constructor(value: number | bigint | boolean | Fr | Buffer) {
    super(value);
  }

  [inspect.custom]() {
    return `Fr<${this.toString()}>`;
  }

  protected modulus() {
    return Fr.MODULUS;
  }

  static random() {
    return random(Fr);
  }

  static zero() {
    return Fr.ZERO;
  }

  static isZero(value: Fr) {
    return value.isZero();
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    return fromBuffer(buffer, Fr);
  }

  static fromBufferReduce(buffer: Buffer) {
    return fromBufferReduce(buffer, Fr);
  }

  /**
   * Creates a Fr instance from a hex string.
   * @param buf - a hex encoded string.
   * @returns the Fr instance
   */
  static fromString(buf: string) {
    return fromHexString(buf, Fr);
  }

  /** Arithmetic */

  add(rhs: Fr) {
    return new Fr((this.toBigInt() + rhs.toBigInt()) % Fr.MODULUS);
  }

  square() {
    return new Fr((this.toBigInt() * this.toBigInt()) % Fr.MODULUS);
  }

  negate() {
    return new Fr(Fr.MODULUS - this.toBigInt());
  }

  sub(rhs: Fr) {
    const result = this.toBigInt() - rhs.toBigInt();
    return new Fr(result < 0 ? result + Fr.MODULUS : result);
  }

  mul(rhs: Fr) {
    return new Fr((this.toBigInt() * rhs.toBigInt()) % Fr.MODULUS);
  }

  div(rhs: Fr) {
    if (rhs.isZero()) {
      throw new Error('Division by zero');
    }

    const bInv = modInverse(rhs.toBigInt());
    return this.mul(bInv);
  }

  // Integer division.
  ediv(rhs: Fr) {
    if (rhs.isZero()) {
      throw new Error('Division by zero');
    }

    return new Fr(this.toBigInt() / rhs.toBigInt());
  }

  /**
   * Computes a square root of the field element.
   * @returns A square root of the field element (null if it does not exist).
   */
  sqrt(): Fr | null {
    const wasm = BarretenbergSync.getSingleton().getWasm();
    wasm.writeMemory(0, this.toBuffer());
    wasm.call('bn254_fr_sqrt', 0, Fr.SIZE_IN_BYTES);
    const isSqrtBuf = Buffer.from(wasm.getMemorySlice(Fr.SIZE_IN_BYTES, Fr.SIZE_IN_BYTES + 1));
    const isSqrt = isSqrtBuf[0] === 1;
    if (!isSqrt) {
      // Field element is not a quadratic residue mod p so it has no square root.
      return null;
    }

    const rootBuf = Buffer.from(wasm.getMemorySlice(Fr.SIZE_IN_BYTES + 1, Fr.SIZE_IN_BYTES * 2 + 1));
    return Fr.fromBuffer(rootBuf);
  }

  toJSON() {
    return {
      type: 'Fr',
      value: this.toString(),
    };
  }
}

// For deserializing JSON.
TypeRegistry.register('Fr', Fr);

/**
 * Branding to ensure fields are not interchangeable types.
 */
export interface Fq {
  /** Brand. */
  _branding: 'Fq';
}

/**
 * Fq field class.
 * @dev This class is used to represent elements of BN254 base field or elements in the scalar field of Grumpkin.
 * (Grumpkin's scalar field corresponds to BN254's base field and vice versa.)
 */
export class Fq extends BaseField {
  static ZERO = new Fq(0n);
  static MODULUS = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47n;
  private static HIGH_SHIFT = BigInt((BaseField.SIZE_IN_BYTES / 2) * 8);
  private static LOW_MASK = (1n << Fq.HIGH_SHIFT) - 1n;

  [inspect.custom]() {
    return `Fq<${this.toString()}>`;
  }

  get lo(): Fr {
    return new Fr(this.toBigInt() & Fq.LOW_MASK);
  }

  get hi(): Fr {
    return new Fr(this.toBigInt() >> Fq.HIGH_SHIFT);
  }

  constructor(value: number | bigint | boolean | Fq | Buffer) {
    super(value);
  }

  protected modulus() {
    return Fq.MODULUS;
  }

  static random() {
    return random(Fq);
  }

  static zero() {
    return Fq.ZERO;
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    return fromBuffer(buffer, Fq);
  }

  static fromBufferReduce(buffer: Buffer) {
    return fromBufferReduce(buffer, Fq);
  }

  /**
   * Creates a Fq instance from a hex string.
   * @param buf - a hex encoded string.
   * @returns the Fq instance
   */
  static fromString(buf: string) {
    return fromHexString(buf, Fq);
  }

  static fromHighLow(high: Fr, low: Fr): Fq {
    return new Fq((high.toBigInt() << Fq.HIGH_SHIFT) + low.toBigInt());
  }

  toJSON() {
    return {
      type: 'Fq',
      value: this.toString(),
    };
  }
}

// For deserializing JSON.
TypeRegistry.register('Fq', Fq);

// Beware: Performance bottleneck below

/**
 * Find the modular inverse of a given element, for BN254 Fr.
 */
function modInverse(b: bigint) {
  const [gcd, x, _] = extendedEuclidean(b, Fr.MODULUS);
  if (gcd != 1n) {
    throw Error('Inverse does not exist');
  }
  // Add modulus if -ve to ensure positive
  return new Fr(x > 0 ? x : x + Fr.MODULUS);
}

/**
 * The extended Euclidean algorithm can be used to find the multiplicative inverse of a field element
 * This is used to perform field division.
 */
function extendedEuclidean(a: bigint, modulus: bigint): [bigint, bigint, bigint] {
  if (a == 0n) {
    return [modulus, 0n, 1n];
  } else {
    const [gcd, x, y] = extendedEuclidean(modulus % a, a);
    return [gcd, y - (modulus / a) * x, x];
  }
}

/**
 * GrumpkinScalar is an Fq.
 * @remarks Called GrumpkinScalar because it is used to represent elements in Grumpkin's scalar field as defined in
 *          the Aztec Protocol Specs.
 */
export type GrumpkinScalar = Fq;
export const GrumpkinScalar = Fq;

/** Wraps a function that returns a buffer so that all results are reduced into a field of the given type. */
export function reduceFn<TInput, TField extends BaseField>(fn: (input: TInput) => Buffer, field: DerivedField<TField>) {
  return (input: TInput) => fromBufferReduce(fn(input), field);
}

/** If we are in test mode, we register a special equality for fields. */
if (process.env.NODE_ENV === 'test') {
  const areFieldsEqual = (a: unknown, b: unknown): boolean | undefined => {
    const isAField = a instanceof BaseField;
    const isBField = b instanceof BaseField;

    if (isAField && isBField) {
      return a.equals(b);
    } else if (isAField === isBField) {
      return undefined;
    } else {
      return false;
    }
  };

  // `addEqualityTesters` doesn't seem to be in the types yet.
  (expect as any).addEqualityTesters([areFieldsEqual]);
}

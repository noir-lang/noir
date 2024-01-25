import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { randomBytes } from '../crypto/random/index.js';
import { BufferReader } from '../serialize/buffer_reader.js';

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
 * If NODE_ENV === 'test', we will always initialize both types to check the modulus.
 * This is also necessary in test environment as a lot of tests just use deep equality to check equality.
 * WARNING: This could lead to a bugs in production that don't reveal in tests, but it's low risk.
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
        throw new Error('Value >= to field modulus.');
      }
    } else if (value instanceof BaseField) {
      this.asBuffer = value.asBuffer;
      this.asBigInt = value.asBigInt;
    } else {
      throw new Error(`Type '${typeof value}' with value '${value}' passed to BaseField ctor.`);
    }

    // Loads of our tests are just doing deep equality rather than calling e.g. toBigInt() first.
    // This ensures the deep equality passes regardless of the internal representation.
    // It also ensures the value range is checked even when initializing as a buffer.
    if (process.env.NODE_ENV === 'test') {
      this.toBuffer();
      this.toBigInt();
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
        throw new Error('Value >= to field modulus.');
      }
    }
    return this.asBigInt;
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

  isZero(): boolean {
    return this.toBuffer().equals(ZERO_BUFFER);
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
function fromBuffer<T extends BaseField>(buffer: Buffer | BufferReader, f: DerivedField<T>) {
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
function fromString<T extends BaseField>(buf: string, f: DerivedField<T>) {
  const buffer = Buffer.from(buf.replace(/^0x/i, ''), 'hex');
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
 */
export class Fr extends BaseField {
  static ZERO = new Fr(0n);
  static MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n;

  constructor(value: number | bigint | boolean | Fr | Buffer) {
    super(value);
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

  static fromBuffer(buffer: Buffer | BufferReader) {
    return fromBuffer(buffer, Fr);
  }

  static fromBufferReduce(buffer: Buffer) {
    return fromBufferReduce(buffer, Fr);
  }

  static fromString(buf: string) {
    return fromString(buf, Fr);
  }

  /** Arithmetic */

  add(rhs: Fr) {
    return new Fr((this.toBigInt() + rhs.toBigInt()) % Fr.MODULUS);
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
}

/**
 * Branding to ensure fields are not interchangeable types.
 */
export interface Fq {
  /** Brand. */
  _branding: 'Fq';
}

/**
 * Fq field class.
 */
export class Fq extends BaseField {
  static ZERO = new Fq(0n);
  static MODULUS = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47n;
  private static HIGH_SHIFT = BigInt((BaseField.SIZE_IN_BYTES / 2) * 8);
  private static LOW_MASK = (1n << Fq.HIGH_SHIFT) - 1n;

  get low(): Fr {
    return new Fr(this.toBigInt() & Fq.LOW_MASK);
  }

  get high(): Fr {
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

  static fromString(buf: string) {
    return fromString(buf, Fq);
  }

  static fromHighLow(high: Fr, low: Fr): Fq {
    return new Fq((high.toBigInt() << Fq.HIGH_SHIFT) + low.toBigInt());
  }
}

// Beware: Performance bottleneck below

/**
 * Find the modular inverse of a given element, for BN254 Fr.
 */
function modInverse(b: bigint) {
  const [gcd, x, _] = extendedEuclidean(b, Fr.MODULUS);
  if (gcd != 1n) {
    throw Error('Inverse does not exist');
  }
  // Add modulus to ensure positive
  return new Fr(x + Fr.MODULUS);
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
 *          the Aztec Yellow Paper.
 */
export type GrumpkinScalar = Fq;
export const GrumpkinScalar = Fq;

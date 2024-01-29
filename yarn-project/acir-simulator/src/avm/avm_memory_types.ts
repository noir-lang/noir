import { Fr } from '@aztec/foundation/fields';

import { strict as assert } from 'assert';

export abstract class MemoryValue {
  public abstract add(rhs: MemoryValue): MemoryValue;
  public abstract sub(rhs: MemoryValue): MemoryValue;
  public abstract mul(rhs: MemoryValue): MemoryValue;
  public abstract div(rhs: MemoryValue): MemoryValue;

  public abstract equals(rhs: MemoryValue): boolean;
  public abstract lt(rhs: MemoryValue): boolean;

  // We need this to be able to build an instance of the subclasses.
  public abstract build(n: bigint): MemoryValue;

  // Use sparingly.
  public abstract toBigInt(): bigint;
}

export abstract class IntegralValue extends MemoryValue {
  public abstract shl(rhs: IntegralValue): IntegralValue;
  public abstract shr(rhs: IntegralValue): IntegralValue;
  public abstract and(rhs: IntegralValue): IntegralValue;
  public abstract or(rhs: IntegralValue): IntegralValue;
  public abstract xor(rhs: IntegralValue): IntegralValue;
  public abstract not(): IntegralValue;
}

// TODO: Optimize calculation of mod, etc. Can only do once per class?
abstract class UnsignedInteger extends IntegralValue {
  private readonly bitmask: bigint;
  private readonly mod: bigint;

  protected constructor(private n: bigint, private bits: bigint) {
    super();
    assert(bits > 0);
    // x % 2^n == x & (2^n - 1)
    this.mod = 1n << bits;
    this.bitmask = this.mod - 1n;
    assert(n < this.mod);
  }

  public abstract build(n: bigint): UnsignedInteger;

  public add(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    return this.build((this.n + rhs.n) & this.bitmask);
  }

  public sub(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    const res: bigint = this.n - rhs.n;
    return this.build(res >= 0 ? res : res + this.mod);
  }

  public mul(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    return this.build((this.n * rhs.n) & this.bitmask);
  }

  public div(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    return this.build(this.n / rhs.n);
  }

  // No sign extension.
  public shr(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    // Note that this.n is > 0 by class invariant.
    return this.build(this.n >> rhs.n);
  }

  public shl(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    return this.build((this.n << rhs.n) & this.bitmask);
  }

  public and(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    return this.build(this.n & rhs.n);
  }

  public or(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    return this.build(this.n | rhs.n);
  }

  public xor(rhs: UnsignedInteger): UnsignedInteger {
    assert(this.bits == rhs.bits);
    return this.build(this.n ^ rhs.n);
  }

  public not(): UnsignedInteger {
    return this.build(~this.n & this.bitmask);
  }

  public equals(rhs: UnsignedInteger): boolean {
    assert(this.bits == rhs.bits);
    return this.n === rhs.n;
  }

  public lt(rhs: UnsignedInteger): boolean {
    assert(this.bits == rhs.bits);
    return this.n < rhs.n;
  }

  public toBigInt(): bigint {
    return this.n;
  }
}

export class Uint8 extends UnsignedInteger {
  constructor(n: number | bigint) {
    super(BigInt(n), 8n);
  }

  public build(n: bigint): Uint8 {
    return new Uint8(n);
  }
}

export class Uint16 extends UnsignedInteger {
  constructor(n: number | bigint) {
    super(BigInt(n), 16n);
  }

  public build(n: bigint): Uint16 {
    return new Uint16(n);
  }
}

export class Uint32 extends UnsignedInteger {
  constructor(n: number | bigint) {
    super(BigInt(n), 32n);
  }

  public build(n: bigint): Uint32 {
    return new Uint32(n);
  }
}

export class Uint64 extends UnsignedInteger {
  constructor(n: number | bigint) {
    super(BigInt(n), 64n);
  }

  public build(n: bigint): Uint64 {
    return new Uint64(n);
  }
}

export class Uint128 extends UnsignedInteger {
  constructor(n: number | bigint) {
    super(BigInt(n), 128n);
  }

  public build(n: bigint): Uint128 {
    return new Uint128(n);
  }
}

export class Field extends MemoryValue {
  public static readonly MODULUS: bigint = Fr.MODULUS;
  private readonly rep: Fr;

  constructor(v: number | bigint | Fr) {
    super();
    this.rep = new Fr(v);
  }

  public build(n: bigint): Field {
    return new Field(n);
  }

  public add(rhs: Field): Field {
    return new Field(this.rep.add(rhs.rep));
  }

  public sub(rhs: Field): Field {
    return new Field(this.rep.sub(rhs.rep));
  }

  public mul(rhs: Field): Field {
    return new Field(this.rep.mul(rhs.rep));
  }

  public div(rhs: Field): Field {
    return new Field(this.rep.div(rhs.rep));
  }

  public equals(rhs: Field): boolean {
    return this.rep.equals(rhs.rep);
  }

  public lt(rhs: Field): boolean {
    return this.rep.lt(rhs.rep);
  }

  public toBigInt(): bigint {
    return this.rep.toBigInt();
  }
}

export enum TypeTag {
  UNINITIALIZED,
  UINT8,
  UINT16,
  UINT32,
  UINT64,
  UINT128,
  FIELD,
  INVALID,
}

// TODO: Consider automatic conversion when getting undefined values.
export class TaggedMemory {
  static readonly MAX_MEMORY_SIZE = 1n << 32n;
  private _mem: MemoryValue[];

  constructor() {
    this._mem = [];
  }

  public get(offset: number): MemoryValue {
    return this.getAs<MemoryValue>(offset);
  }

  public getAs<T>(offset: number): T {
    assert(offset < TaggedMemory.MAX_MEMORY_SIZE);
    const word = this._mem[offset];
    return word as T;
  }

  public getSlice(offset: number, size: number): MemoryValue[] {
    assert(offset < TaggedMemory.MAX_MEMORY_SIZE);
    return this._mem.slice(offset, offset + size);
  }

  public getSliceTags(offset: number, size: number): TypeTag[] {
    assert(offset < TaggedMemory.MAX_MEMORY_SIZE);
    return this._mem.slice(offset, offset + size).map(TaggedMemory.getTag);
  }

  public set(offset: number, v: MemoryValue) {
    assert(offset < TaggedMemory.MAX_MEMORY_SIZE);
    this._mem[offset] = v;
  }

  public setSlice(offset: number, vs: MemoryValue[]) {
    assert(offset < TaggedMemory.MAX_MEMORY_SIZE);
    this._mem.splice(offset, vs.length, ...vs);
  }

  public getTag(offset: number): TypeTag {
    return TaggedMemory.getTag(this._mem[offset]);
  }

  // TODO: this might be slow, but I don't want to have the types know of their tags.
  // It might be possible to have a map<Prototype, TypeTag>.
  public static getTag(v: MemoryValue | undefined): TypeTag {
    let tag = TypeTag.INVALID;

    if (v === undefined) {
      tag = TypeTag.UNINITIALIZED;
    } else if (v instanceof Field) {
      tag = TypeTag.FIELD;
    } else if (v instanceof Uint8) {
      tag = TypeTag.UINT8;
    } else if (v instanceof Uint16) {
      tag = TypeTag.UINT16;
    } else if (v instanceof Uint32) {
      tag = TypeTag.UINT32;
    } else if (v instanceof Uint64) {
      tag = TypeTag.UINT64;
    } else if (v instanceof Uint128) {
      tag = TypeTag.UINT128;
    }

    return tag;
  }

  // Truncates the value to fit the type.
  public static integralFromTag(v: bigint, tag: TypeTag): IntegralValue {
    v = BigInt(v); // FIXME: not sure why this cast is needed, but this errors otherwise
    switch (tag) {
      case TypeTag.UINT8:
        return new Uint8(v & ((1n << 8n) - 1n));
      case TypeTag.UINT16:
        return new Uint16(v & ((1n << 16n) - 1n));
      case TypeTag.UINT32:
        return new Uint32(v & ((1n << 32n) - 1n));
      case TypeTag.UINT64:
        return new Uint64(v & ((1n << 64n) - 1n));
      case TypeTag.UINT128:
        return new Uint128(v & ((1n << 128n) - 1n));
      default:
        throw new Error(`${TypeTag[tag]} is not a valid integral type.`);
    }
  }
}

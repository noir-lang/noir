import {
  Field,
  MeteredTaggedMemory,
  TaggedMemory,
  Uint8,
  Uint16,
  Uint32,
  Uint64,
  Uint128,
} from './avm_memory_types.js';

describe('TaggedMemory', () => {
  it('Elements should be undefined after construction', () => {
    const mem = new TaggedMemory();
    expect(mem.get(10)).toBe(undefined);
  });

  it(`Should set and get integral types`, () => {
    const mem = new TaggedMemory();
    mem.set(10, new Uint8(5));
    expect(mem.get(10)).toStrictEqual(new Uint8(5));
  });

  it(`Should set and get field elements`, () => {
    const mem = new TaggedMemory();
    mem.set(10, new Field(5));
    expect(mem.get(10)).toStrictEqual(new Field(5));
  });

  it(`Should fail getSlice on unset elements`, () => {
    const mem = new TaggedMemory();

    mem.set(10, new Field(10));
    mem.set(12, new Field(12));

    expect(() => mem.getSlice(10, /*size=*/ 4)).toThrow(/size/);
  });

  it(`Should set and get slices`, () => {
    const mem = new TaggedMemory();
    const val = [new Field(5), new Field(6)];

    mem.setSlice(10, val);

    expect(mem.getSlice(10, /*size=*/ 2)).toStrictEqual(val);
  });
});

describe('MeteredTaggedMemory', () => {
  let mem: MeteredTaggedMemory;

  beforeEach(() => {
    mem = new MeteredTaggedMemory(new TaggedMemory());
  });

  it(`Counts reads`, () => {
    mem.get(10);
    mem.getAs(20);
    expect(mem.reset()).toEqual({ reads: 2, writes: 0 });
  });

  it(`Counts reading slices`, () => {
    const val = [new Field(5), new Field(6), new Field(7)];
    mem.setSlice(10, val);
    mem.reset();

    mem.getSlice(10, 3);
    mem.getSliceAs(11, 2);
    expect(mem.reset()).toEqual({ reads: 5, writes: 0 });
  });

  it(`Counts writes`, () => {
    mem.set(10, new Uint8(5));
    expect(mem.reset()).toEqual({ reads: 0, writes: 1 });
  });

  it(`Counts writing slices`, () => {
    mem.setSlice(10, [new Field(5), new Field(6)]);
    expect(mem.reset()).toEqual({ reads: 0, writes: 2 });
  });

  it(`Clears stats`, () => {
    mem.get(10);
    mem.set(20, new Uint8(5));
    expect(mem.reset()).toEqual({ reads: 1, writes: 1 });
    expect(mem.reset()).toEqual({ reads: 0, writes: 0 });
  });

  it(`Asserts stats`, () => {
    mem.get(10);
    mem.set(20, new Uint8(5));
    expect(() => mem.assert({ reads: 1, writes: 1 })).not.toThrow();
  });

  it(`Throws on failed stat assertion`, () => {
    mem.get(10);
    expect(() => mem.assert({ reads: 1, writes: 1 })).toThrow();
  });
});

type IntegralClass = typeof Uint8 | typeof Uint16 | typeof Uint32 | typeof Uint64 | typeof Uint128;
describe.each([Uint8, Uint16, Uint32, Uint64, Uint128])('Integral Types', (clsValue: IntegralClass) => {
  describe(`${clsValue.name}`, () => {
    it(`Should construct a new ${clsValue.name} from a number`, () => {
      const x = new clsValue(5);
      expect(x.toBigInt()).toStrictEqual(5n);
    });

    it(`Should construct a new ${clsValue.name} from a bigint`, () => {
      const x = new clsValue(5n);
      expect(x.toBigInt()).toStrictEqual(5n);
    });

    it(`Should build a new ${clsValue.name}`, () => {
      const x = new clsValue(5);
      const newX = x.build(10n);
      expect(newX).toStrictEqual(new clsValue(10n));
    });

    it(`Should add two ${clsValue.name} correctly`, () => {
      const a = new clsValue(5);
      const b = new clsValue(10);
      const result = a.add(b);
      expect(result).toStrictEqual(new clsValue(15n));
    });

    it(`Should subtract two ${clsValue.name} correctly`, () => {
      const a = new clsValue(10);
      const b = new clsValue(5);
      const result = a.sub(b);
      expect(result).toStrictEqual(new clsValue(5n));
    });

    it(`Should multiply two ${clsValue.name} correctly`, () => {
      const a = new clsValue(5);
      const b = new clsValue(10);
      const result = a.mul(b);
      expect(result).toStrictEqual(new clsValue(50n));
    });

    it(`Should divide two ${clsValue.name} correctly`, () => {
      const a = new clsValue(10);
      const b = new clsValue(5);
      const result = a.div(b);
      expect(result).toStrictEqual(new clsValue(2n));
    });

    it(`Should shift right ${clsValue.name} correctly`, () => {
      const uintA = new clsValue(10);
      const result = uintA.shr(new clsValue(1n));
      expect(result).toEqual(new clsValue(5n));
    });

    it(`Should shift left ${clsValue.name} correctly`, () => {
      const uintA = new clsValue(10);
      const result = uintA.shl(new clsValue(1n));
      expect(result).toEqual(new clsValue(20n));
    });

    it(`Should and two ${clsValue.name} correctly`, () => {
      const uintA = new clsValue(10);
      const uintB = new clsValue(5);
      const result = uintA.and(uintB);
      expect(result).toEqual(new clsValue(0n));
    });

    it(`Should or two ${clsValue.name} correctly`, () => {
      const uintA = new clsValue(10);
      const uintB = new clsValue(5);
      const result = uintA.or(uintB);
      expect(result).toEqual(new clsValue(15n));
    });

    it(`Should xor two ${clsValue.name} correctly`, () => {
      const uintA = new clsValue(10);
      const uintB = new clsValue(5);
      const result = uintA.xor(uintB);
      expect(result).toEqual(new clsValue(15n));
    });

    it(`Should check equality of two ${clsValue.name} correctly`, () => {
      const a = new clsValue(5);
      const b = new clsValue(5);
      const c = new clsValue(10);
      expect(a.equals(b)).toBe(true);
      expect(a.equals(c)).toBe(false);
    });

    it(`Should check if one ${clsValue.name} is less than another correctly`, () => {
      const a = new clsValue(5);
      const b = new clsValue(10);
      expect(a.lt(b)).toBe(true);
      expect(b.lt(a)).toBe(false);
    });
  });
});

describe('Field', () => {
  it(`Should build a new Field`, () => {
    const field = new Field(5);
    const newField = field.build(10n);
    expect(newField.toBigInt()).toStrictEqual(10n);
  });

  it(`Should add two Fields correctly`, () => {
    const field1 = new Field(5);
    const field2 = new Field(10);
    const result = field1.add(field2);
    expect(result).toStrictEqual(new Field(15n));
  });

  it(`Should subtract two Fields correctly`, () => {
    const field1 = new Field(10);
    const field2 = new Field(5);
    const result = field1.sub(field2);
    expect(result).toStrictEqual(new Field(5n));
  });

  it(`Should multiply two Fields correctly`, () => {
    const field1 = new Field(5);
    const field2 = new Field(10);
    const result = field1.mul(field2);
    expect(result).toStrictEqual(new Field(50n));
  });

  it(`Should divide two Fields correctly`, () => {
    const field1 = new Field(10);
    const field2 = new Field(5);
    const result = field1.div(field2);
    expect(result).toStrictEqual(new Field(2n));
  });

  it(`Should check equality of two Fields correctly`, () => {
    const field1 = new Field(5);
    const field2 = new Field(5);
    const field3 = new Field(10);
    expect(field1.equals(field2)).toBe(true);
    expect(field1.equals(field3)).toBe(false);
  });

  it(`Should check if one Field is less than another correctly`, () => {
    const field1 = new Field(5);
    const field2 = new Field(10);
    expect(field1.lt(field2)).toBe(true);
    expect(field2.lt(field1)).toBe(false);
  });

  it(`Should convert Field to BigInt correctly`, () => {
    const field = new Field(5);
    expect(field.toBigInt()).toStrictEqual(5n);
  });
});

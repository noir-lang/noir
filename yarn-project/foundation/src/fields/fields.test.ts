import { Fr, GrumpkinScalar } from './fields.js';

describe('GrumpkinScalar Serialization', () => {
  // Test case for GrumpkinScalar.fromHighLow
  it('fromHighLow should serialize and deserialize correctly', () => {
    const original = GrumpkinScalar.random();
    const high = original.hi;
    const low = original.lo;

    const deserialized = GrumpkinScalar.fromHighLow(high, low);

    // Check if the deserialized instance is equal to the original
    expect(deserialized).toEqual(original);
  });

  // Test case for GrumpkinScalar.fromBuffer
  it('fromBuffer should serialize and deserialize correctly', () => {
    const original = GrumpkinScalar.random();
    const buffer = original.toBuffer();
    const deserialized = GrumpkinScalar.fromBuffer(buffer);

    // Check if the deserialized instance is equal to the original
    expect(deserialized).toEqual(original);
  });

  // Test case for GrumpkinScalar.fromString
  it('fromString should serialize and deserialize correctly', () => {
    const original = GrumpkinScalar.random();
    const hexString = original.toString();
    const deserialized = GrumpkinScalar.fromString(hexString);

    // Check if the deserialized instance is equal to the original
    expect(deserialized).toEqual(original);

    // Note odd number of digits
    const arbitraryString = '123';
    const arbitraryHexString = '0x123';
    const expectedBigInt = 291n;

    expect(GrumpkinScalar.fromString(arbitraryString).toBigInt()).toEqual(expectedBigInt);
    expect(GrumpkinScalar.fromString(arbitraryHexString).toBigInt()).toEqual(expectedBigInt);

    const incorrectlyFormattedString = '12xx34xx45';

    expect(() => GrumpkinScalar.fromString(incorrectlyFormattedString).toBigInt()).toThrow();
  });

  // Test case for GrumpkinScalar.toBuffer
  it('toBuffer should serialize and deserialize correctly', () => {
    const original = GrumpkinScalar.random();
    const buffer = original.toBuffer();
    const deserialized = GrumpkinScalar.fromBuffer(buffer);

    // Check if the deserialized instance is equal to the original
    expect(deserialized).toEqual(original);
  });

  // Test case for GrumpkinScalar.toString
  it('toString should serialize and deserialize correctly', () => {
    const original = GrumpkinScalar.random();
    const hexString = original.toString();
    const deserialized = GrumpkinScalar.fromString(hexString);

    // Check if the deserialized instance is equal to the original
    expect(deserialized).toEqual(original);
  });
});

describe('Bn254 arithmetic', () => {
  describe('Addition', () => {
    it('Low Boundary', () => {
      // 0 + -1 = -1
      const a = Fr.ZERO;
      const b = new Fr(Fr.MODULUS - 1n);
      const expected = new Fr(Fr.MODULUS - 1n);

      const actual = a.add(b);
      expect(actual).toEqual(expected);
    });

    it('High Boundary', () => {
      // -1 + 1 = 0
      const a = new Fr(Fr.MODULUS - 1n);
      const b = new Fr(1);
      const expected = Fr.ZERO;

      const actual = a.add(b);
      expect(actual).toEqual(expected);
    });

    it('Performs addition correctly', () => {
      const a = new Fr(2);
      const b = new Fr(3);
      const expected = new Fr(5);

      const actual = a.add(b);
      expect(actual).toEqual(expected);
    });
  });

  describe('Subtraction', () => {
    it('Low Boundary', () => {
      // 0 - 1 = -1
      const a = new Fr(0);
      const b = new Fr(1);
      const expected = new Fr(Fr.MODULUS - 1n);

      const actual = a.sub(b);
      expect(actual).toEqual(expected);
    });

    it('High Bonudary', () => {
      // -1 - (-1) = 0
      const a = new Fr(Fr.MODULUS - 1n);
      const b = new Fr(Fr.MODULUS - 1n);

      const actual = a.sub(b);
      expect(actual).toEqual(Fr.ZERO);
    });

    it('Performs subtraction correctly', () => {
      const a = new Fr(10);
      const b = new Fr(5);
      const expected = new Fr(5);

      const actual = a.sub(b);
      expect(actual).toEqual(expected);
    });
  });

  describe('Multiplication', () => {
    it('Identity', () => {
      const a = new Fr(Fr.MODULUS - 1n);
      const b = new Fr(1);
      const expected = new Fr(Fr.MODULUS - 1n);

      const actual = a.mul(b);
      expect(actual).toEqual(expected);
    });

    it('Performs multiplication correctly', () => {
      const a = new Fr(2);
      const b = new Fr(3);
      const expected = new Fr(6);

      const actual = a.mul(b);
      expect(actual).toEqual(expected);
    });

    it('High Boundary', () => {
      const a = new Fr(Fr.MODULUS - 1n);
      const b = new Fr(Fr.MODULUS / 2n);
      const expected = new Fr(10944121435919637611123202872628637544274182200208017171849102093287904247809n);

      const actual = a.mul(b);
      expect(actual).toEqual(expected);
    });
  });

  describe('Division', () => {
    it('Should succeed when mod inverse is -ve', () => {
      const a = new Fr(2);
      const b = new Fr(3);

      const actual = a.div(b);
      expect(actual.mul(b)).toEqual(a);
    });

    it('Should succeed when mod inverse is +ve', () => {
      const a = new Fr(10);
      const b = new Fr(5);
      const expected = new Fr(2);

      const actual = a.div(b);
      expect(actual.mul(b)).toEqual(a);
      expect(actual).toEqual(expected);
    });

    it('Should not allow a division by 0', () => {
      const a = new Fr(10);
      const b = Fr.ZERO;

      expect(() => a.div(b)).toThrow();
    });
  });

  describe('Comparison', () => {
    it.each([
      [new Fr(5), new Fr(10), -1],
      [new Fr(10), new Fr(5), 1],
      [new Fr(5), new Fr(5), 0],
      [new Fr(0), new Fr(Fr.MODULUS - 1n), -1],
      [new Fr(Fr.MODULUS - 1n), new Fr(0), 1],
      [Fr.ZERO, Fr.ZERO, 0],
      [Fr.zero(), Fr.ZERO, 0],
    ])('Should compare field elements correctly', (a, b, expected) => {
      expect(a.cmp(b)).toEqual(expected);
    });
  });
});

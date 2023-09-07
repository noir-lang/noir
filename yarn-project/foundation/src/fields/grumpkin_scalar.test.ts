import { GrumpkinScalar } from './grumpkin_scalar.js';

describe('GrumpkinScalar Serialization', () => {
  // Test case for GrumpkinScalar.fromHighLow
  it('fromHighLow should serialize and deserialize correctly', () => {
    const original = GrumpkinScalar.random();
    const high = original.high;
    const low = original.low;

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

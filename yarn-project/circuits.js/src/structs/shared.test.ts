import { EcdsaSignature } from './shared.js';

describe('shared', () => {
  it('serializes signature with v to fields', () => {
    const signature = EcdsaSignature.random();
    const asFields = signature.toFields(true);
    const parts = asFields.map(fr => fr.toBuffer().subarray(1));
    const reassembled = EcdsaSignature.fromBuffer(Buffer.concat(parts));
    expect(reassembled).toEqual(signature);
  });

  it('serializes signature without v to fields', () => {
    const signature = EcdsaSignature.random();
    const asFields = signature.toFields(false);
    const parts = asFields.map(fr => fr.toBuffer().subarray(1));
    const reassembled = EcdsaSignature.fromBuffer(Buffer.concat(parts));
    expect(reassembled.r).toEqual(signature.r);
    expect(reassembled.s).toEqual(signature.s);
    expect(reassembled.v).toEqual(Buffer.alloc(1));
  });
});

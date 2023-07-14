import { randomBytes } from '../crypto/index.js';
import { Fr } from '../fields/fields.js';
import {
  deserializeArrayFromVector,
  deserializeBufferFromVector,
  deserializeField,
  deserializeUInt32,
  serializeBufferArrayToVector,
  serializeBufferToVector,
} from './index.js';

describe('serialize', () => {
  it('serialize buffer to vector and deserialize it back', () => {
    const data = randomBytes(32);
    const vector = serializeBufferToVector(data);
    expect(vector.length).toBe(36);

    const recovered = deserializeBufferFromVector(vector);
    expect(recovered.elem).toEqual(data);
    expect(recovered.adv).toEqual(4 + 32);

    const paddedVector = Buffer.concat([randomBytes(10), vector, randomBytes(20)]);
    const recovered2 = deserializeBufferFromVector(paddedVector, 10);
    expect(recovered2.elem).toEqual(data);
    expect(recovered2.adv).toEqual(4 + 32);
  });

  it('deserialize uint32', () => {
    const uintBuf = Buffer.alloc(4);
    uintBuf.writeUInt32BE(19, 0);

    const recovered = deserializeUInt32(uintBuf);
    expect(recovered.elem).toBe(19);
    expect(recovered.adv).toBe(4);

    const paddedBuf = Buffer.concat([randomBytes(10), uintBuf, randomBytes(20)]);
    const recovered2 = deserializeUInt32(paddedBuf, 10);
    expect(recovered2.elem).toBe(19);
    expect(recovered2.adv).toBe(4);
  });

  it('deserialize field', () => {
    const field = Fr.random();

    const recovered = deserializeField(field.toBuffer());
    expect(recovered.elem).toEqual(field);
    expect(recovered.adv).toBe(32);

    const paddedBuf = Buffer.concat([randomBytes(10), field.toBuffer(), randomBytes(20)]);
    const recovered2 = deserializeField(paddedBuf, 10);
    expect(recovered2.elem).toEqual(field);
    expect(recovered2.adv).toBe(32);
  });

  it('serialize buffer array to vector and deserialize it back', () => {
    // Array of uint32
    const uintArr = [7, 13, 16];
    const uintBufArr = uintArr.map(num => {
      const uintBuf = Buffer.alloc(4);
      uintBuf.writeUInt32BE(num, 0);
      return uintBuf;
    });
    const uintArrVec = serializeBufferArrayToVector(uintBufArr);
    expect(uintArrVec.length).toBe(4 + 4 * 3);

    const recoveredUintArr = deserializeArrayFromVector(deserializeUInt32, uintArrVec);
    expect(recoveredUintArr.elem).toEqual(uintArr);
    expect(recoveredUintArr.adv).toEqual(4 + 4 * 3);

    const paddedUintArrVec = Buffer.concat([randomBytes(10), uintArrVec, randomBytes(20)]);
    const recoveredUintArr2 = deserializeArrayFromVector(deserializeUInt32, paddedUintArrVec, 10);
    expect(recoveredUintArr2.elem).toEqual(uintArr);
    expect(recoveredUintArr2.adv).toEqual(4 + 4 * 3);

    // Array of field
    const fieldArr = [Fr.random(), Fr.random(), Fr.random()];
    const fieldArrVec = serializeBufferArrayToVector(fieldArr.map(fr => fr.toBuffer()));
    expect(fieldArrVec.length).toBe(4 + 32 * 3);

    const recoveredFieldArr = deserializeArrayFromVector(deserializeField, fieldArrVec);
    expect(recoveredFieldArr.elem).toEqual(fieldArr);
    expect(recoveredFieldArr.adv).toEqual(4 + 32 * 3);

    const paddedFieldVec = Buffer.concat([randomBytes(10), fieldArrVec, randomBytes(20)]);
    const recoveredFieldArr2 = deserializeArrayFromVector(deserializeField, paddedFieldVec, 10);
    expect(recoveredFieldArr2.elem).toEqual(fieldArr);
    expect(recoveredFieldArr2.adv).toEqual(4 + 32 * 3);
  });
});

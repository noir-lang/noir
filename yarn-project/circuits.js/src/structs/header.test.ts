import { makeHeader } from '../tests/factories.js';
import { Header } from './header.js';

describe('Header', () => {
  it('serializes to buffer and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makeHeader(randomInt, undefined);
    const buffer = expected.toBuffer();
    const res = Header.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to field array and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makeHeader(randomInt, undefined);

    const fieldArray = expected.toFieldArray();
    const res = Header.fromFieldArray(fieldArray);
    expect(res).toEqual(expected);
  });

  it('computes hash', () => {
    const seed = 9870243;
    const header = makeHeader(seed, undefined);
    const hash = header.hash();
    expect(hash).toMatchSnapshot();
  });
});

import { makeHeader } from '../tests/factories.js';
import { Header } from './header.js';

describe('Header', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makeHeader(randomInt, undefined);
    const buffer = expected.toBuffer();
    const res = Header.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});

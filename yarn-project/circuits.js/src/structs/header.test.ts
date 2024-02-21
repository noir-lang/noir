import { HEADER_LENGTH } from '../constants.gen.js';
import { makeHeader } from '../tests/factories.js';
import { Header } from './header.js';

describe('Header', () => {
  let header: Header;

  beforeAll(() => {
    const randomInt = Math.floor(Math.random() * 1000);
    header = makeHeader(randomInt, undefined);
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = header.toBuffer();
    const res = Header.fromBuffer(buffer);
    expect(res).toEqual(header);
  });

  it('serializes to field array and deserializes it back', () => {
    const fieldArray = header.toFields();
    const res = Header.fromFields(fieldArray);
    expect(res).toEqual(header);
  });

  it('computes hash', () => {
    const seed = 9870243;
    const header = makeHeader(seed, undefined);
    const hash = header.hash();
    expect(hash).toMatchSnapshot();
  });

  it('number of fields matches constant', () => {
    const fields = header.toFields();
    expect(fields.length).toBe(HEADER_LENGTH);
  });

  it('computes empty hash', () => {
    const header = Header.empty();
    const hash = header.hash();
    expect(hash).toMatchSnapshot();

    // Value used in empty_hash test in header.nr
    // console.log("hash", hash.toString());
  });
});

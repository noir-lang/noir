import { randomBytes } from './index.js';

describe('random', () => {
  it('randomBytes returns a filled byte array', () => {
    const data = randomBytes(32);
    expect(data.length).toEqual(32);
    let identical = true;
    for (let i = 1; i < data.length; ++i) {
      identical = identical && data[i] == data[i - 1];
    }
    expect(identical).toEqual(false);
  });
});

import { Aztec } from './index.js';

describe('Aztec', () => {
  it('Initialise Aztec', () => {
    expect(() => new Aztec()).not.toThrow();
  });
});

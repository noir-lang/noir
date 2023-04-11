import { UnverifiedData } from './unverified_data.js';

describe('UnverifiedData', () => {
  it('can encode UnverifiedData to buffer and back', () => {
    const unverifiedData = UnverifiedData.random(42);

    const buffer = unverifiedData.toBuffer();
    const recovered = UnverifiedData.fromBuffer(buffer);

    expect(recovered).toEqual(unverifiedData);
  });
});

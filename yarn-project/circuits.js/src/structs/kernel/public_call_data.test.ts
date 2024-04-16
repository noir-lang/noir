import { makePublicCallData } from '../../tests/factories.js';
import { PublicCallData } from './public_call_data.js';

describe('PublicCallData', () => {
  it('PublicCallData after serialization and deserialization is equal to the original', () => {
    const original = makePublicCallData(123, true);
    const serialized = PublicCallData.fromBuffer(original.toBuffer());
    expect(original).toEqual(serialized);
  });
});

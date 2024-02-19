import { makeAccumulatedData, makeFinalAccumulatedData } from '../../tests/factories.js';
import { CombinedAccumulatedData, PrivateAccumulatedRevertibleData } from './combined_accumulated_data.js';

describe('CombinedAccumulatedData', () => {
  it('Data after serialization and deserialization is equal to the original', () => {
    const original = makeAccumulatedData();
    const afterSerialization = CombinedAccumulatedData.fromBuffer(original.toBuffer());
    expect(original).toEqual(afterSerialization);
  });
});

describe('FinalAccumulatedData', () => {
  it('Data after serialization and deserialization is equal to the original', () => {
    const original = makeFinalAccumulatedData();
    const afterSerialization = PrivateAccumulatedRevertibleData.fromBuffer(original.toBuffer());
    expect(original).toEqual(afterSerialization);
  });
});

import { makePreviousKernelData } from '../../tests/factories.js';
import { PreviousKernelData } from './previous_kernel_data.js';

describe('PreviousKernelData', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const expected = makePreviousKernelData();
    const buffer = expected.toBuffer();
    const res = PreviousKernelData.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});

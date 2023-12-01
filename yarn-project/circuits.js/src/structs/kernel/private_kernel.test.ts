import { makePrivateKernelInputsInner } from '../../tests/factories.js';
import { PrivateKernelInputsInner } from './private_kernel.js';

describe('PrivateKernel', function () {
  it(`serializes PrivateKernelInputsInner to buffer and deserializes it back`, () => {
    const expected = makePrivateKernelInputsInner();
    const buffer = expected.toBuffer();
    const res = PrivateKernelInputsInner.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});

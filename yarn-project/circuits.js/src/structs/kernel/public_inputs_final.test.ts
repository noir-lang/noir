import { makePrivateKernelPublicInputsFinal } from '../../tests/factories.js';
import { PrivateKernelPublicInputsFinal } from './public_inputs_final.js';

describe('PrivateKernelPublicInputsFinal', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const expected = makePrivateKernelPublicInputsFinal();
    const buffer = expected.toBuffer();
    const res = PrivateKernelPublicInputsFinal.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});

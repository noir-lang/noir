import { makeKernelPublicInputs } from '../../tests/factories.js';
import { KernelCircuitPublicInputs } from './public_inputs.js';

describe('KernelCircuitPublicInputs', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const expected = makeKernelPublicInputs();
    const buffer = expected.toBuffer();
    const res = KernelCircuitPublicInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});

import { makePrivateKernelTailCircuitPublicInputs } from '../../tests/factories.js';
import { PrivateKernelTailCircuitPublicInputs } from './private_kernel_tail_circuit_public_inputs.js';

describe('PrivateKernelTailCircuitPublicInputs', () => {
  it('Data for public after serialization and deserialization is equal to the original', () => {
    const original = makePrivateKernelTailCircuitPublicInputs(123, true);
    const serialized = PrivateKernelTailCircuitPublicInputs.fromBuffer(original.toBuffer());
    expect(original).toEqual(serialized);
  });

  it('Data for rollup after serialization and deserialization is equal to the original', () => {
    const original = makePrivateKernelTailCircuitPublicInputs(123, false);
    const serialized = PrivateKernelTailCircuitPublicInputs.fromBuffer(original.toBuffer());
    expect(original).toEqual(serialized);
  });
});

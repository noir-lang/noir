import { makePublicKernelCircuitPrivateInputs } from '../../tests/factories.js';
import { PublicKernelCircuitPrivateInputs } from './public_kernel_circuit_private_inputs.js';

describe('PublicKernelCircuitPrivateInputs', () => {
  it('PublicKernelCircuitPrivateInputs after serialization and deserialization is equal to the original', () => {
    const original = makePublicKernelCircuitPrivateInputs(123);
    const serialized = PublicKernelCircuitPrivateInputs.fromBuffer(original.toBuffer());
    expect(original).toEqual(serialized);
  });

  it('PublicKernelCircuitPrivateInputs after clone is equal to the original', () => {
    const original = makePublicKernelCircuitPrivateInputs(123);
    const serialized = original.clone();
    expect(original).toEqual(serialized);
    expect(original).not.toBe(serialized);
  });
});

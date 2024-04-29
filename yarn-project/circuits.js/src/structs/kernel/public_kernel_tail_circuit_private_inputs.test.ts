import { makePublicKernelTailCircuitPrivateInputs } from '../../tests/factories.js';
import { PublicKernelTailCircuitPrivateInputs } from './public_kernel_tail_circuit_private_inputs.js';

describe('PublicKernelTailCircuitPrivateInputs', () => {
  it('PublicKernelTailCircuitPrivateInputs after serialization and deserialization is equal to the original', () => {
    const original = makePublicKernelTailCircuitPrivateInputs(123);
    const serialized = PublicKernelTailCircuitPrivateInputs.fromBuffer(original.toBuffer());
    expect(original).toEqual(serialized);
  });

  it('PublicKernelTailCircuitPrivateInputs after clone is equal to the original', () => {
    const original = makePublicKernelTailCircuitPrivateInputs(123);
    const serialized = original.clone();
    expect(original).toEqual(serialized);
    expect(original).not.toBe(serialized);
  });

  it('serializes to string and back', () => {
    const original = makePublicKernelTailCircuitPrivateInputs(123);
    const str = original.toString();
    const deserialized = PublicKernelTailCircuitPrivateInputs.fromString(str);
    expect(original).toEqual(deserialized);
  });
});

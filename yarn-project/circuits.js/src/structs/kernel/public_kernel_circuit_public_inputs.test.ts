import { makePublicKernelCircuitPublicInputs } from '../../tests/factories.js';
import { PublicKernelCircuitPublicInputs } from './public_kernel_circuit_public_inputs.js';

describe('PublicKernelCircuitPublicInputs', () => {
  it('PublicKernelCircuitPublicInputs after serialization and deserialization is equal to the original', () => {
    const original = makePublicKernelCircuitPublicInputs(123);
    const serialized = PublicKernelCircuitPublicInputs.fromBuffer(original.toBuffer());
    expect(original).toEqual(serialized);
  });

  it('PublicKernelCircuitPublicInputs after clone is equal to the original', () => {
    const original = makePublicKernelCircuitPublicInputs(123);
    const serialized = original.clone();
    expect(original).toEqual(serialized);
    expect(original).not.toBe(serialized);
  });

  it('serializes to and deserializes from a string', () => {
    const original = makePublicKernelCircuitPublicInputs(123);
    const serialized = PublicKernelCircuitPublicInputs.fromString(original.toString());
    expect(original).toEqual(serialized);
  });
});

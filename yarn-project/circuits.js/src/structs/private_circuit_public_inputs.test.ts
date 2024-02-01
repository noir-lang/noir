import { makePrivateCircuitPublicInputs } from '../tests/factories.js';
import { PrivateCircuitPublicInputs } from './private_circuit_public_inputs.js';

describe('PrivateCircuitPublicInputs', () => {
  it('serializes to buffer and back', () => {
    const target = makePrivateCircuitPublicInputs(100);
    const buffer = target.toBuffer();
    const result = PrivateCircuitPublicInputs.fromBuffer(buffer);
    expect(result).toEqual(target);
  });

  it('serializes to fields and back', () => {
    const target = makePrivateCircuitPublicInputs(100);
    const fields = target.toFields();
    const result = PrivateCircuitPublicInputs.fromFields(fields);
    expect(result).toEqual(target);
  });

  it(`initializes an empty PrivateCircuitPublicInputs`, () => {
    const target = PrivateCircuitPublicInputs.empty();
    expect(target.isEmpty()).toBe(true);
  });
});

import { PrivateCircuitPublicInputs } from './private_circuit_public_inputs.js';

describe('PrivateCircuitPublicInputs', () => {
  it(`initializes an empty PrivateCircuitPublicInputs`, () => {
    const target = PrivateCircuitPublicInputs.empty();
    expect(target.isEmpty()).toBe(true);
  });
});

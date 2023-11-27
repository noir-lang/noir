import { PublicCircuitPublicInputs } from './public_circuit_public_inputs.js';

describe('PublicCircuitPublicInputs', () => {
  it(`initializes an empty PrivateCircuitPublicInputs`, () => {
    const target = PublicCircuitPublicInputs.empty();
    expect(target.isEmpty()).toBe(true);
  });
});

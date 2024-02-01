import { makePublicCircuitPublicInputs } from '../tests/factories.js';
import { PublicCircuitPublicInputs } from './public_circuit_public_inputs.js';

describe('PublicCircuitPublicInputs', () => {
  it('serializes to field array and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makePublicCircuitPublicInputs(randomInt, undefined);

    const fieldArray = expected.toFields();
    const res = PublicCircuitPublicInputs.fromFields(fieldArray);
    expect(res).toEqual(expected);
  });

  it(`initializes an empty PrivateCircuitPublicInputs`, () => {
    const target = PublicCircuitPublicInputs.empty();
    expect(target.isEmpty()).toBe(true);
  });
});

import { PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH } from '../constants.gen.js';
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

  it('number of fields matches constant', () => {
    const target = makePublicCircuitPublicInputs(327);
    const fields = target.toFields();
    expect(fields.length).toBe(PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH);
  });

  it('hash matches snapshot', () => {
    const target = makePublicCircuitPublicInputs(327);
    const hash = target.hash();
    expect(hash).toMatchSnapshot();
  });

  it('computes empty item hash', () => {
    const item = PublicCircuitPublicInputs.empty();
    const hash = item.hash();
    expect(hash).toMatchSnapshot();

    // Value used in empty_hash test in public_circuit_public_inputs.nr
    // console.log("hash", hash.toString());
  });
});

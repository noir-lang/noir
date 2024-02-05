import { PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH } from '../constants.gen.js';
import { makePrivateCircuitPublicInputs } from '../tests/factories.js';
import { PrivateCircuitPublicInputs } from './private_circuit_public_inputs.js';

describe('PrivateCircuitPublicInputs', () => {
  let inputs: PrivateCircuitPublicInputs;

  beforeAll(() => {
    const randomInt = Math.floor(Math.random() * 1000);
    inputs = makePrivateCircuitPublicInputs(randomInt);
  });

  it('serializes to buffer and back', () => {
    const buffer = inputs.toBuffer();
    const result = PrivateCircuitPublicInputs.fromBuffer(buffer);
    expect(result).toEqual(inputs);
  });

  it('serializes to fields and back', () => {
    const fields = inputs.toFields();
    const result = PrivateCircuitPublicInputs.fromFields(fields);
    expect(result).toEqual(inputs);
  });

  it(`initializes an empty PrivateCircuitPublicInputs`, () => {
    const target = PrivateCircuitPublicInputs.empty();
    expect(target.isEmpty()).toBe(true);
  });

  it('number of fields matches constant', () => {
    const fields = inputs.toFields();
    expect(fields.length).toBe(PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH);
  });

  it('hash matches snapshot', () => {
    const target = makePrivateCircuitPublicInputs(327);
    const hash = target.hash();
    expect(hash).toMatchSnapshot();
  });

  it('computes empty inputs hash', () => {
    const inputs = PrivateCircuitPublicInputs.empty();
    const hash = inputs.hash();
    expect(hash).toMatchSnapshot();

    // Value used in empty_hash test in private_circuit_public_inputs.nr
    // console.log("hash", hash.toString());
  });
});

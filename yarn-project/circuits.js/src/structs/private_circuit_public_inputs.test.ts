import { randomInt } from '@aztec/foundation/crypto';
import { setupCustomSnapshotSerializers, updateInlineTestData } from '@aztec/foundation/testing';

import { PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH } from '../constants.gen.js';
import { makePrivateCircuitPublicInputs } from '../tests/factories.js';
import { PrivateCircuitPublicInputs } from './private_circuit_public_inputs.js';

describe('PrivateCircuitPublicInputs', () => {
  let inputs: PrivateCircuitPublicInputs;

  beforeAll(() => {
    setupCustomSnapshotSerializers(expect);
    inputs = makePrivateCircuitPublicInputs(randomInt(1000));
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

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/noir-protocol-circuits/crates/types/src/abis/private_circuit_public_inputs.nr',
      'test_data_empty_hash',
      hash.toString(),
    );
  });
});

import { randomInt } from '@aztec/foundation/crypto';
import { setupCustomSnapshotSerializers, updateInlineTestData } from '@aztec/foundation/testing';

import { PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH } from '../constants.gen.js';
import { makePublicCircuitPublicInputs } from '../tests/factories.js';
import { PublicCircuitPublicInputs } from './public_circuit_public_inputs.js';

describe('PublicCircuitPublicInputs', () => {
  setupCustomSnapshotSerializers(expect);
  it('serializes to field array and deserializes it back', () => {
    const expected = makePublicCircuitPublicInputs(randomInt(1000), undefined);

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

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/noir-protocol-circuits/crates/types/src/abis/public_circuit_public_inputs.nr',
      'test_data_empty_hash',
      hash.toString(),
    );
  });
});

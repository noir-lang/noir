import { expectSerializeToMatchSnapshot } from '../tests/expectSerialize.js';
import { makePublicCircuitPublicInputs } from '../tests/factories.js';

describe('basic PublicCircuitPublicInputs serialization', () => {
  it(`serializes a trivial PublicCircuitPublicInputs and prints it`, async () => {
    // Test the data case: writing (mostly) sequential numbers
    await expectSerializeToMatchSnapshot(
      makePublicCircuitPublicInputs().toBuffer(),
      'abis__test_roundtrip_serialize_public_circuit_public_inputs',
    );
  });
});

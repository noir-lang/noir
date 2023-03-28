import { expectSerializeToMatchSnapshot } from '../tests/expectSerialize.js';
import { makePrivateCircuitPublicInputs } from '../tests/factories.js';

describe('basic PrivateCircuitPublicInputs serialization', () => {
  it(`serializes a trivial PrivateCircuitPublicInputs and prints it`, async () => {
    // Test the data case: writing (mostly) sequential numbers
    await expectSerializeToMatchSnapshot(
      makePrivateCircuitPublicInputs().toBuffer(),
      'abis__test_roundtrip_serialize_private_circuit_public_inputs',
    );
  });
});

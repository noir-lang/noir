import { expectReserializeToMatchObject, expectSerializeToMatchSnapshot } from '../../tests/expectSerialize.js';
import { makeRootRollupInputs, makeRootRollupPublicInputs } from '../../tests/factories.js';
import { RootRollupPublicInputs } from './root_rollup.js';

describe('structs/root_rollup', () => {
  it(`serializes a RootRollupInput and prints it`, async () => {
    await expectSerializeToMatchSnapshot(
      makeRootRollupInputs().toBuffer(),
      'abis__test_roundtrip_serialize_root_rollup_inputs',
    );
  });

  it(`serializes a RootRollupPublicInputs and prints it`, async () => {
    await expectSerializeToMatchSnapshot(
      makeRootRollupPublicInputs().toBuffer(),
      'abis__test_roundtrip_serialize_root_rollup_public_inputs',
    );
  });

  it(`serializes a RootRollupPublicInputs and deserializes it back`, async () => {
    await expectReserializeToMatchObject(
      makeRootRollupPublicInputs(),
      'abis__test_roundtrip_reserialize_root_rollup_public_inputs',
      RootRollupPublicInputs.fromBuffer,
    );
  });
});

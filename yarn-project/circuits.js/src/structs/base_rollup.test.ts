import { expectReserializeToMatchObject, expectSerializeToMatchSnapshot } from '../tests/expectSerialize.js';
import {
  fr,
  makeAppendOnlyTreeSnapshot,
  makeBaseRollupPublicInputs,
  makeConstantBaseRollupData,
  makePreviousKernelData,
} from '../tests/factories.js';
import { range } from '../utils/jsUtils.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { BaseRollupInputs, BaseRollupPublicInputs, NullifierLeafPreimage } from './base_rollup.js';
import {
  CONTRACT_TREE_HEIGHT,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_DATA_TREE_HEIGHT,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
} from './constants.js';
import { PreviousKernelData } from './kernel.js';
import { MembershipWitness } from './shared.js';

describe('structs/base_rollup', () => {
  it(`serializes and prints BaseRollupInputs`, async () => {
    const kernelData: [PreviousKernelData, PreviousKernelData] = [
      makePreviousKernelData(0x100),
      makePreviousKernelData(0x200),
    ];

    const startPrivateDateTreeSnapshot = makeAppendOnlyTreeSnapshot(0x100);
    const startNullifierTreeSnapshot = makeAppendOnlyTreeSnapshot(0x200);
    const startContractTreeSnapshot = makeAppendOnlyTreeSnapshot(0x300);

    const lowNullifierLeafPreimages = range(2 * KERNEL_NEW_NULLIFIERS_LENGTH, 0x1000).map(
      x => new NullifierLeafPreimage(fr(x), fr(x + 0x100), x + 0x200),
    );

    const lowNullifierMembershipWitness = range(2 * KERNEL_NEW_NULLIFIERS_LENGTH, 0x2000).map(x =>
      MembershipWitness.mock(NULLIFIER_TREE_HEIGHT, x),
    );

    const newCommitmentsSubtreeSiblingPath = range(
      PRIVATE_DATA_TREE_HEIGHT - BaseRollupInputs.PRIVATE_DATA_SUBTREE_HEIGHT,
      0x3000,
    ).map(x => fr(x));
    const newNullifiersSubtreeSiblingPath = range(
      NULLIFIER_TREE_HEIGHT - BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT,
      0x4000,
    ).map(x => fr(x));
    const newContractsSubtreeSiblingPath = range(
      CONTRACT_TREE_HEIGHT - BaseRollupInputs.CONTRACT_SUBTREE_HEIGHT,
      0x5000,
    ).map(x => fr(x));

    const historicPrivateDataTreeRootMembershipWitnesses: BaseRollupInputs['historicPrivateDataTreeRootMembershipWitnesses'] =
      [
        MembershipWitness.mock(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT, 0x6000),
        MembershipWitness.mock(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT, 0x7000),
      ];

    const historicContractsTreeRootMembershipWitnesses: BaseRollupInputs['historicContractsTreeRootMembershipWitnesses'] =
      [
        MembershipWitness.mock(CONTRACT_TREE_ROOTS_TREE_HEIGHT, 0x8000),
        MembershipWitness.mock(CONTRACT_TREE_ROOTS_TREE_HEIGHT, 0x9000),
      ];

    const constants = makeConstantBaseRollupData(0x100);

    const baseRollupInputs = BaseRollupInputs.from({
      kernelData,
      startPrivateDateTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      lowNullifierLeafPreimages,
      lowNullifierMembershipWitness,
      newCommitmentsSubtreeSiblingPath,
      newNullifiersSubtreeSiblingPath,
      newContractsSubtreeSiblingPath,
      historicPrivateDataTreeRootMembershipWitnesses,
      historicContractsTreeRootMembershipWitnesses,
      constants,
    });

    const wasm = await CircuitsWasm.new();
    await expectSerializeToMatchSnapshot(
      baseRollupInputs.toBuffer(),
      'abis__test_roundtrip_serialize_base_rollup_inputs',
      wasm,
    );
  });

  it(`serializes and prints BaseRollupPublicInputs`, async () => {
    const baseRollupPublicInputs = makeBaseRollupPublicInputs();

    await expectSerializeToMatchSnapshot(
      baseRollupPublicInputs.toBuffer(),
      'abis__test_roundtrip_serialize_base_rollup_public_inputs',
    );
  });

  it(`serializes and deserializes BaseRollupPublicInputs`, async () => {
    const baseRollupPublicInputs = makeBaseRollupPublicInputs();

    await expectReserializeToMatchObject(
      baseRollupPublicInputs,
      'abis__test_roundtrip_reserialize_base_rollup_public_inputs',
      BaseRollupPublicInputs.fromBuffer,
    );
  });
});

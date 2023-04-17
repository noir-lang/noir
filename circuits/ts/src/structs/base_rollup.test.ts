import {
  expectReserializeToMatchObject,
  expectSerializeToMatchSnapshot,
} from "../tests/expectSerialize.js";
import {
  fr,
  makeAppendOnlyTreeSnapshot,
  makeBaseRollupPublicInputs,
  makeConstantBaseRollupData,
  makePreviousKernelData,
} from "../tests/factories.js";
import { writeGlobalVerifierReferenceString } from "../tests/writeGlobalVerifierReferenceString.js";
import { range } from "../utils/jsUtils.js";
import { CircuitsWasm } from "../wasm/circuits_wasm.js";
import {
  BaseRollupInputs,
  BaseRollupPublicInputs,
  NullifierLeafPreimage,
} from "./base_rollup.js";
import {
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
} from "./constants.js";
import { PreviousKernelData } from "./kernel.js";
import { MembershipWitness } from "./shared.js";

describe("structs/base_rollup", () => {
  it(`serializes and prints BaseRollupInputs`, async () => {
    const kernelData: [PreviousKernelData, PreviousKernelData] = [
      makePreviousKernelData(0x100),
      makePreviousKernelData(0x200),
    ];

    const startNullifierTreeSnapshot = makeAppendOnlyTreeSnapshot(0x100);

    const lowNullifierLeafPreimages = range(
      2 * KERNEL_NEW_NULLIFIERS_LENGTH,
      0x1000
    ).map((x) => new NullifierLeafPreimage(fr(x), fr(x + 0x100), x + 0x200));

    const lowNullifierMembershipWitness = range(
      2 * KERNEL_NEW_NULLIFIERS_LENGTH,
      0x2000
    ).map((x) => MembershipWitness.mock(NULLIFIER_TREE_HEIGHT, x));

    const historicPrivateDataTreeRootMembershipWitnesses: BaseRollupInputs["historicPrivateDataTreeRootMembershipWitnesses"] =
      [
        MembershipWitness.mock(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT, 0x3000),
        MembershipWitness.mock(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT, 0x4000),
      ];

    const historicContractsTreeRootMembershipWitnesses: BaseRollupInputs["historicContractsTreeRootMembershipWitnesses"] =
      [
        MembershipWitness.mock(CONTRACT_TREE_ROOTS_TREE_HEIGHT, 0x5000),
        MembershipWitness.mock(CONTRACT_TREE_ROOTS_TREE_HEIGHT, 0x6000),
      ];

    const constants = makeConstantBaseRollupData(0x100);

    const proverId = fr(0x42);

    const baseRollupInputs = BaseRollupInputs.from({
      kernelData,
      startNullifierTreeSnapshot,
      lowNullifierLeafPreimages,
      lowNullifierMembershipWitness,
      historicPrivateDataTreeRootMembershipWitnesses,
      historicContractsTreeRootMembershipWitnesses,
      constants,
      proverId,
    });

    const wasm = await CircuitsWasm.new();
    await writeGlobalVerifierReferenceString(
      wasm,
      /* example circuit size */ 100
    );
    await expectSerializeToMatchSnapshot(
      baseRollupInputs.toBuffer(),
      "abis__test_roundtrip_serialize_base_rollup_inputs",
      wasm
    );
  });

  it(`serializes and prints BaseRollupPublicInputs`, async () => {
    const baseRollupPublicInputs = makeBaseRollupPublicInputs();

    await expectSerializeToMatchSnapshot(
      baseRollupPublicInputs.toBuffer(),
      "abis__test_roundtrip_serialize_base_rollup_public_inputs"
    );
  });

  it(`serializes and deserializes BaseRollupPublicInputs`, async () => {
    const baseRollupPublicInputs = makeBaseRollupPublicInputs();

    await expectReserializeToMatchObject(
      baseRollupPublicInputs,
      "abis__test_roundtrip_reserialize_base_rollup_public_inputs",
      BaseRollupPublicInputs.fromBuffer
    );
  });
});

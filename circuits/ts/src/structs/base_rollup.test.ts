import { expectSerializeToMatchSnapshot } from "../tests/expectSerializeToMatchSnapshot.js";
import { fr, makePreviousKernelData } from "../tests/factories.js";
import { writeGlobalVerifierReferenceString } from "../tests/writeGlobalVerifierReferenceString";
import { range } from "../utils/jsUtils.js";
import { CircuitsWasm } from "../wasm/circuits_wasm.js";
import {
  AppendOnlyTreeSnapshot,
  BaseRollupInputs,
  ConstantBaseRollupData,
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
  it(`serializes and prints object`, async () => {
    const kernelData: [PreviousKernelData, PreviousKernelData] = [
      makePreviousKernelData(0x100),
      makePreviousKernelData(0x200),
    ];

    const startNullifierTreeSnapshot = new AppendOnlyTreeSnapshot(
      fr(0x100),
      0x100
    );

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

    const constants = ConstantBaseRollupData.from({
      startTreeOfHistoricPrivateDataTreeRootsSnapshot:
        new AppendOnlyTreeSnapshot(fr(0x100), 0x100),
      startTreeOfHistoricContractTreeRootsSnapshot: new AppendOnlyTreeSnapshot(
        fr(0x200),
        0x200
      ),
      treeOfHistoricL1ToL2MsgTreeRootsSnapshot: new AppendOnlyTreeSnapshot(
        fr(0x300),
        0x300
      ),
      privateKernelVkTreeRoot: fr(0x400),
      publicKernelVkTreeRoot: fr(0x500),
      baseRollupVkHash: fr(0x600),
      mergeRollupVkHash: fr(0x700),
    });

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
});

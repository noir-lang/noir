import {
  AppendOnlyTreeSnapshot,
  BaseRollupPublicInputs,
  ConstantBaseRollupData,
} from "../structs/base_rollup.js";
import {
  EMITTED_EVENTS_LENGTH,
  KERNEL_L1_MSG_STACK_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  VK_TREE_HEIGHT,
} from "../structs/constants.js";
import { FunctionData } from "../structs/function_data.js";
import {
  AccumulatedData,
  ConstantData,
  NewContractData,
  OldTreeRoots,
  OptionallyRevealedData,
  PreviousKernelData,
  PrivateKernelPublicInputs,
} from "../structs/kernel.js";
import {
  AffineElement,
  AggregationObject,
  ComposerType,
  EthAddress,
  Fq,
  Fr,
  MembershipWitness,
  UInt8Vector,
  RollupTypes,
} from "../structs/shared.js";
import { ContractDeploymentData, TxContext } from "../structs/tx.js";
import {
  CommitmentMap,
  G1AffineElement,
  VerificationKey,
} from "../structs/verification_key.js";
import { range } from "../utils/jsUtils.js";
import { numToUInt32BE } from "../utils/serialize.js";

export function makeTxContext(seed: number): TxContext {
  const deploymentData = new ContractDeploymentData(
    fr(seed),
    fr(seed + 1),
    fr(seed + 2),
    makeEthAddress(seed + 3)
  );
  return new TxContext(false, false, true, deploymentData);
}

export function makeOldTreeRoots(seed: number): OldTreeRoots {
  return new OldTreeRoots(fr(seed), fr(seed + 1), fr(seed + 2), fr(seed + 3));
}

export function makeConstantData(seed = 1): ConstantData {
  return new ConstantData(makeOldTreeRoots(seed), makeTxContext(seed + 4));
}

export function makeAccumulatedData(seed = 1): AccumulatedData {
  return new AccumulatedData(
    makeAggregationObject(seed),
    fr(seed + 12),
    range(KERNEL_NEW_COMMITMENTS_LENGTH, seed + 0x100).map(fr),
    range(KERNEL_NEW_NULLIFIERS_LENGTH, seed + 0x200).map(fr),
    range(KERNEL_PRIVATE_CALL_STACK_LENGTH, seed + 0x300).map(fr),
    range(KERNEL_PUBLIC_CALL_STACK_LENGTH, seed + 0x400).map(fr),
    range(KERNEL_L1_MSG_STACK_LENGTH, seed + 0x500).map(fr),
    range(KERNEL_NEW_CONTRACTS_LENGTH, seed + 0x600).map(makeNewContractData),
    range(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, seed + 0x700).map(
      makeOptionallyRevealedData
    )
  );
}

export function makeNewContractData(seed = 1): NewContractData {
  return new NewContractData(fr(seed), makeEthAddress(seed + 1), fr(seed + 2));
}

export function makeOptionallyRevealedData(seed = 1): OptionallyRevealedData {
  return new OptionallyRevealedData(
    fr(seed),
    new FunctionData(seed + 1, true, true),
    range(EMITTED_EVENTS_LENGTH, seed + 0x100).map((x) => fr(x)),
    fr(seed + 2),
    makeEthAddress(seed + 3),
    true,
    false,
    true,
    false
  );
}

export function makeAggregationObject(seed = 1): AggregationObject {
  return new AggregationObject(
    new AffineElement(new Fq(seed), new Fq(seed + 1)),
    new AffineElement(new Fq(seed + 0x100), new Fq(seed + 0x101)),
    range(4, seed + 2).map(fr),
    range(6, seed + 6)
  );
}

export function makePrivateKernelPublicInputs(
  seed = 1
): PrivateKernelPublicInputs {
  return new PrivateKernelPublicInputs(
    makeAccumulatedData(seed),
    makeConstantData(seed + 0x100),
    true
  );
}

export function makeDynamicSizeBuffer(size: number, fill: number) {
  return new UInt8Vector(Buffer.alloc(size, fill));
}

export function makeMembershipWitness<N extends number>(
  size: number,
  start: number
): MembershipWitness<N> {
  return new MembershipWitness(size, start, range(size, start).map(fr));
}

export function makeVerificationKey(): VerificationKey {
  return new VerificationKey(
    ComposerType.STANDARD,
    101, // arbitrary
    102, // arbitrary,
    new CommitmentMap({
      A: new G1AffineElement(fr(0x200), fr(0x300)),
    }),
    /* recursive proof */ true,
    range(5, 400)
  );
}
export function makePreviousKernelData(seed = 1): PreviousKernelData {
  return new PreviousKernelData(
    makePrivateKernelPublicInputs(seed),
    makeDynamicSizeBuffer(16, seed + 0x80),
    makeVerificationKey(),
    0x42,
    range(VK_TREE_HEIGHT, 0x1000).map(fr)
  );
}

export function makeConstantBaseRollupData(seed = 1): ConstantBaseRollupData {
  return ConstantBaseRollupData.from({
    startTreeOfHistoricPrivateDataTreeRootsSnapshot:
      makeAppendOnlyTreeSnapshot(seed),
    startTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(
      seed + 0x100
    ),
    treeOfHistoricL1ToL2MsgTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(
      seed + 0x200
    ),
    privateKernelVkTreeRoot: fr(seed + 0x301),
    publicKernelVkTreeRoot: fr(seed + 0x302),
    baseRollupVkHash: fr(seed + 0x303),
    mergeRollupVkHash: fr(seed + 0x304),
  });
}

export function makeAppendOnlyTreeSnapshot(seed = 1): AppendOnlyTreeSnapshot {
  return new AppendOnlyTreeSnapshot(fr(seed), seed);
}

export function makeEthAddress(seed = 1): EthAddress {
  return new EthAddress(Buffer.alloc(20, seed));
}

export function makeBaseRollupPublicInputs(seed = 0) {
  return new BaseRollupPublicInputs(
    RollupTypes.Base,
    makeAggregationObject(seed + 0x100),
    makeConstantBaseRollupData(seed + 0x200),
    makeAppendOnlyTreeSnapshot(seed + 0x300),
    makeAppendOnlyTreeSnapshot(seed + 0x400),
    fr(seed + 0x501),
    fr(seed + 0x502),
    fr(seed + 0x503),
    fr(seed + 0x601),
    fr(seed + 0x602),
    fr(seed + 0x603),
    fr(seed + 0x604),
    fr(seed + 0x605)
  );
}

/**
 * Test only. Easy to identify big endian field serialize.
 * @param num - The number.
 * @param bufferSize - The buffer size.
 * @returns The buffer.
 */
export function fr(n: number) {
  return new Fr(numToUInt32BE(n, 32));
}

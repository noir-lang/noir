import { AztecAddress, EthAddress, Fq, Fr } from '@aztec/foundation';
import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CallContext,
  CombinedAccumulatedData,
  CombinedConstantData,
  CombinedHistoricTreeRoots,
  ConstantBaseRollupData,
  KernelCircuitPublicInputs,
  MergeRollupInputs,
  NewContractData,
  NullifierLeafPreimage,
  OptionallyRevealedData,
  PreviousKernelData,
  PreviousRollupData,
  PrivateCallData,
  PrivateCircuitPublicInputs,
  PrivateKernelInputs,
  PrivateHistoricTreeRoots,
  RootRollupInputs,
  RootRollupPublicInputs,
  StateRead,
  StateTransition,
  PublicDataWrite,
  PublicDataRead,
} from '../index.js';
import { AggregationObject } from '../structs/aggregation_object.js';
import { PrivateCallStackItem } from '../structs/call_stack_item.js';
import {
  ARGS_LENGTH,
  CONTRACT_TREE_HEIGHT,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  EMITTED_EVENTS_LENGTH,
  FUNCTION_TREE_HEIGHT,
  KERNEL_L1_MSG_STACK_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  L1_MSG_STACK_LENGTH,
  NEW_COMMITMENTS_LENGTH,
  NEW_NULLIFIERS_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_CALL_STACK_LENGTH,
  PRIVATE_DATA_TREE_HEIGHT,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
  PUBLIC_CALL_STACK_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  RETURN_VALUES_LENGTH,
  ROLLUP_VK_TREE_HEIGHT,
  STATE_READS_LENGTH,
  STATE_TRANSITIONS_LENGTH,
  VK_TREE_HEIGHT,
} from '../structs/constants.js';
import { FunctionData } from '../structs/function_data.js';
import { MembershipWitness } from '../structs/membership_witness.js';
import { AppendOnlyTreeSnapshot } from '../structs/rollup/append_only_tree_snapshot.js';
import { AffineElement, ComposerType, EcdsaSignature, RollupTypes, UInt8Vector } from '../structs/shared.js';
import { ContractDeploymentData, TxContext } from '../structs/tx_context.js';
import { SignedTxRequest, TxRequest } from '../structs/tx_request.js';
import { CommitmentMap, G1AffineElement, VerificationKey } from '../structs/verification_key.js';
import { range } from '../utils/jsUtils.js';
import { numToUInt32BE } from '../utils/serialize.js';

export function makeTxContext(seed: number): TxContext {
  const deploymentData = new ContractDeploymentData(fr(seed), fr(seed + 1), fr(seed + 2), makeEthAddress(seed + 3));
  return new TxContext(false, false, true, deploymentData);
}

export function makePrivateHistoricTreeRoots(seed: number): PrivateHistoricTreeRoots {
  return new PrivateHistoricTreeRoots(fr(seed), fr(seed + 1), fr(seed + 2), fr(seed + 3));
}

export function makeCombinedHistoricTreeRoots(seed: number): CombinedHistoricTreeRoots {
  return new CombinedHistoricTreeRoots(makePrivateHistoricTreeRoots(seed), fr(seed + 4));
}

export function makeConstantData(seed = 1): CombinedConstantData {
  return new CombinedConstantData(makeCombinedHistoricTreeRoots(seed), makeTxContext(seed + 4));
}

export function makeSelector(seed: number) {
  const buffer = Buffer.alloc(4);
  buffer.writeUInt32BE(seed, 0);
  return buffer;
}

export function makePublicDataWrite(seed = 1) {
  return new PublicDataWrite(fr(seed), fr(seed + 1));
}

export function makePublicDataRead(seed = 1) {
  return new PublicDataRead(fr(seed), fr(seed + 1));
}

export function makeStateTransition(seed = 1) {
  return new StateTransition(fr(seed), fr(seed + 1), fr(seed + 2));
}

export function makeStateRead(seed = 1) {
  return new StateRead(fr(seed), fr(seed + 1));
}

export function makeAccumulatedData(seed = 1): CombinedAccumulatedData {
  return new CombinedAccumulatedData(
    makeAggregationObject(seed),
    fr(seed + 12),
    fr(seed + 13),
    range(KERNEL_NEW_COMMITMENTS_LENGTH, seed + 0x100).map(fr),
    range(KERNEL_NEW_NULLIFIERS_LENGTH, seed + 0x200).map(fr),
    range(KERNEL_PRIVATE_CALL_STACK_LENGTH, seed + 0x300).map(fr),
    range(KERNEL_PUBLIC_CALL_STACK_LENGTH, seed + 0x400).map(fr),
    range(KERNEL_L1_MSG_STACK_LENGTH, seed + 0x500).map(fr),
    range(KERNEL_NEW_CONTRACTS_LENGTH, seed + 0x600).map(makeNewContractData),
    range(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, seed + 0x700).map(makeOptionallyRevealedData),
    range(STATE_TRANSITIONS_LENGTH, seed + 0x800).map(makePublicDataWrite),
  );
}

export function makeNewContractData(seed = 1): NewContractData {
  return new NewContractData(makeAztecAddress(seed), makeEthAddress(seed + 1), fr(seed + 2));
}

export function makeOptionallyRevealedData(seed = 1): OptionallyRevealedData {
  return new OptionallyRevealedData(
    fr(seed),
    new FunctionData(makeSelector(seed + 1), true, true),
    range(EMITTED_EVENTS_LENGTH, seed + 0x100).map(x => fr(x)),
    fr(seed + 2),
    makeEthAddress(seed + 3),
    true,
    false,
    true,
    false,
  );
}

export function makeAggregationObject(seed = 1): AggregationObject {
  return new AggregationObject(
    new AffineElement(new Fq(BigInt(seed)), new Fq(BigInt(seed + 1))),
    new AffineElement(new Fq(BigInt(seed + 0x100)), new Fq(BigInt(seed + 0x101))),
    range(4, seed + 2).map(fr),
    range(6, seed + 6),
  );
}

export function makeKernelPublicInputs(seed = 1): KernelCircuitPublicInputs {
  return new KernelCircuitPublicInputs(makeAccumulatedData(seed), makeConstantData(seed + 0x100), true);
}

export function makeDynamicSizeBuffer(size: number, fill: number) {
  return new UInt8Vector(Buffer.alloc(size, fill));
}

export function makeMembershipWitness<N extends number>(size: number, start: number): MembershipWitness<N> {
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
    range(5, 400),
  );
}

export function makePreviousKernelData(seed = 1): PreviousKernelData {
  return new PreviousKernelData(
    makeKernelPublicInputs(seed),
    makeDynamicSizeBuffer(16, seed + 0x80),
    makeVerificationKey(),
    0x42,
    range(VK_TREE_HEIGHT, 0x1000).map(fr),
  );
}

export function makePrivateKernelInputs(seed = 1): PrivateKernelInputs {
  return new PrivateKernelInputs(
    makeSignedTxRequest(seed),
    makePreviousKernelData(seed + 0x1000),
    makePrivateCallData(seed + 0x2000),
  );
}

export function makeSignedTxRequest(seed = 1): SignedTxRequest {
  return new SignedTxRequest(makeTxRequest(seed), makeEcdsaSignature(seed + 0x200));
}

export function makeTxRequest(seed = 1): TxRequest {
  return TxRequest.from({
    from: makeAztecAddress(seed),
    to: makeAztecAddress(seed + 0x10),
    functionData: new FunctionData(makeSelector(seed + 0x100), true, true),
    args: range(ARGS_LENGTH, seed + 0x200).map(x => fr(x)),
    nonce: fr(seed + 0x300),
    txContext: makeTxContext(seed + 0x400),
    chainId: fr(seed + 0x500),
  });
}

export function makePrivateCallData(seed = 1): PrivateCallData {
  return PrivateCallData.from({
    callStackItem: makeCallStackItem(seed),
    privateCallStackPreimages: range(PRIVATE_CALL_STACK_LENGTH, seed + 0x10).map(makeCallStackItem),
    proof: makeDynamicSizeBuffer(16, seed + 0x50),
    vk: makeVerificationKey(),
    functionLeafMembershipWitness: makeMembershipWitness(FUNCTION_TREE_HEIGHT, seed + 0x30),
    contractLeafMembershipWitness: makeMembershipWitness(CONTRACT_TREE_HEIGHT, seed + 0x20),
    portalContractAddress: makeEthAddress(seed + 0x40),
    acirHash: fr(seed + 0x60),
  });
}

export function makeCallStackItem(seed = 1): PrivateCallStackItem {
  return new PrivateCallStackItem(
    makeAztecAddress(seed),
    new FunctionData(makeSelector(seed + 0x1), true, true),
    makePrivateCircuitPublicInputs(seed + 0x10),
  );
}

export function makePrivateCircuitPublicInputs(seed = 0): PrivateCircuitPublicInputs {
  return PrivateCircuitPublicInputs.from({
    callContext: new CallContext(
      makeAztecAddress(seed + 1),
      makeAztecAddress(seed + 2),
      new EthAddress(numToUInt32BE(seed + 3, /* eth address is 20 bytes */ 20)),
      true,
      true,
      true,
    ),
    args: range(ARGS_LENGTH, seed + 0x100).map(fr),
    emittedEvents: range(EMITTED_EVENTS_LENGTH, seed + 0x200).map(fr), // TODO not in spec
    returnValues: range(RETURN_VALUES_LENGTH, seed + 0x300).map(fr),
    newCommitments: range(NEW_COMMITMENTS_LENGTH, seed + 0x400).map(fr),
    newNullifiers: range(NEW_NULLIFIERS_LENGTH, seed + 0x500).map(fr),
    privateCallStack: range(PRIVATE_CALL_STACK_LENGTH, seed + 0x600).map(fr),
    publicCallStack: range(PUBLIC_CALL_STACK_LENGTH, seed + 0x700).map(fr),
    l1MsgStack: range(L1_MSG_STACK_LENGTH, seed + 0x800).map(fr),
    historicContractTreeRoot: fr(seed + 0x900), // TODO not in spec
    historicPrivateDataTreeRoot: fr(seed + 0x1000),
    historicPrivateNullifierTreeRoot: fr(seed + 0x1100), // TODO not in spec
    contractDeploymentData: makeContractDeploymentData(),
  });
}

export function makeContractDeploymentData(seed = 1) {
  return new ContractDeploymentData(fr(seed), fr(seed + 1), fr(seed + 2), new EthAddress(numToUInt32BE(seed + 3, 20)));
}

export function makeConstantBaseRollupData(seed = 1): ConstantBaseRollupData {
  return ConstantBaseRollupData.from({
    startTreeOfHistoricPrivateDataTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed),
    startTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x100),
    treeOfHistoricL1ToL2MsgTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x200),
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

export function makeBytes(size = 32, seed = 1): Buffer {
  return Buffer.alloc(size, seed);
}

export function makeAztecAddress(seed = 1): AztecAddress {
  return new AztecAddress(fr(seed).toBuffer());
}

export function makeEcdsaSignature(seed = 1): EcdsaSignature {
  return new EcdsaSignature(Buffer.alloc(32, seed), Buffer.alloc(32, seed + 1));
}

export function makeBaseRollupPublicInputs(seed = 0) {
  return new BaseOrMergeRollupPublicInputs(
    RollupTypes.Base,
    new Fr(0n),
    makeAggregationObject(seed + 0x100),
    makeConstantBaseRollupData(seed + 0x200),
    makeAppendOnlyTreeSnapshot(seed + 0x300),
    makeAppendOnlyTreeSnapshot(seed + 0x400),
    makeAppendOnlyTreeSnapshot(seed + 0x500),
    makeAppendOnlyTreeSnapshot(seed + 0x600),
    makeAppendOnlyTreeSnapshot(seed + 0x700),
    makeAppendOnlyTreeSnapshot(seed + 0x800),
    makeAppendOnlyTreeSnapshot(seed + 0x900),
    makeAppendOnlyTreeSnapshot(seed + 0x1000),
    [fr(seed + 0x901), fr(seed + 0x902)],
  );
}

export function makePreviousBaseRollupData(seed = 0) {
  return new PreviousRollupData(
    makeBaseRollupPublicInputs(seed),
    makeDynamicSizeBuffer(16, seed + 0x50),
    makeVerificationKey(),
    seed + 0x110,
    makeMembershipWitness(ROLLUP_VK_TREE_HEIGHT, seed + 0x120),
  );
}

export function makeRootRollupInputs(seed = 0) {
  return new RootRollupInputs(
    [makePreviousBaseRollupData(seed), makePreviousBaseRollupData(seed + 0x1000)],
    range(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT, 0x2000).map(fr),
    range(CONTRACT_TREE_ROOTS_TREE_HEIGHT, 0x2100).map(fr),
  );
}

export function makeRootRollupPublicInputs(seed = 0) {
  return RootRollupPublicInputs.from({
    startContractTreeSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x100),
    startNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x200),
    startPrivateDataTreeSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x300),
    startTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x400),
    startTreeOfHistoricPrivateDataTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x500),
    endContractTreeSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x600),
    endNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x700),
    endPrivateDataTreeSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x800),
    endTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x900),
    endTreeOfHistoricPrivateDataTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x1000),
    endAggregationObject: makeAggregationObject(seed + 0x1100),
    calldataHash: [new Fr(0n), new Fr(0n)],
  });
}

export function makeMergeRollupInputs(seed = 0) {
  return new MergeRollupInputs([makePreviousBaseRollupData(seed), makePreviousBaseRollupData(seed + 0x1000)]);
}

export function makeBaseRollupInputs(seed = 0) {
  const kernelData: [PreviousKernelData, PreviousKernelData] = [
    makePreviousKernelData(seed + 0x100),
    makePreviousKernelData(seed + 0x200),
  ];

  const startPrivateDataTreeSnapshot = makeAppendOnlyTreeSnapshot(seed + 0x100);
  const startNullifierTreeSnapshot = makeAppendOnlyTreeSnapshot(seed + 0x200);
  const startContractTreeSnapshot = makeAppendOnlyTreeSnapshot(seed + 0x300);
  const startPublicDataTreeSnapshot = makeAppendOnlyTreeSnapshot(seed + 0x400);

  const lowNullifierLeafPreimages = range(2 * KERNEL_NEW_NULLIFIERS_LENGTH, seed + 0x1000).map(
    x => new NullifierLeafPreimage(fr(x), fr(x + 0x100), x + 0x200),
  );

  const lowNullifierMembershipWitness = range(2 * KERNEL_NEW_NULLIFIERS_LENGTH, seed + 0x2000).map(x =>
    makeMembershipWitness(NULLIFIER_TREE_HEIGHT, x),
  );

  const newCommitmentsSubtreeSiblingPath = range(
    PRIVATE_DATA_TREE_HEIGHT - BaseRollupInputs.PRIVATE_DATA_SUBTREE_HEIGHT,
    seed + 0x3000,
  ).map(x => fr(x));

  const newNullifiersSubtreeSiblingPath = range(
    NULLIFIER_TREE_HEIGHT - BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT,
    seed + 0x4000,
  ).map(x => fr(x));

  const newContractsSubtreeSiblingPath = range(
    CONTRACT_TREE_HEIGHT - BaseRollupInputs.CONTRACT_SUBTREE_HEIGHT,
    seed + 0x5000,
  ).map(x => fr(x));

  const newStateTransitionsSiblingPath = range(2 * STATE_TRANSITIONS_LENGTH, seed + 0x6000).map(x =>
    makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, x),
  );

  const historicPrivateDataTreeRootMembershipWitnesses: BaseRollupInputs['historicPrivateDataTreeRootMembershipWitnesses'] =
    [
      makeMembershipWitness(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT, seed + 0x6000),
      makeMembershipWitness(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT, seed + 0x7000),
    ];

  const historicContractsTreeRootMembershipWitnesses: BaseRollupInputs['historicContractsTreeRootMembershipWitnesses'] =
    [
      makeMembershipWitness(CONTRACT_TREE_ROOTS_TREE_HEIGHT, seed + 0x8000),
      makeMembershipWitness(CONTRACT_TREE_ROOTS_TREE_HEIGHT, seed + 0x9000),
    ];

  const constants = makeConstantBaseRollupData(0x100);

  return BaseRollupInputs.from({
    kernelData,
    startPrivateDataTreeSnapshot,
    startNullifierTreeSnapshot,
    startContractTreeSnapshot,
    startPublicDataTreeSnapshot,
    lowNullifierLeafPreimages,
    lowNullifierMembershipWitness,
    newCommitmentsSubtreeSiblingPath,
    newNullifiersSubtreeSiblingPath,
    newContractsSubtreeSiblingPath,
    newStateTransitionsSiblingPaths: newStateTransitionsSiblingPath,
    historicPrivateDataTreeRootMembershipWitnesses,
    historicContractsTreeRootMembershipWitnesses,
    constants,
  });
}

/**
 * Test only. Easy to identify big endian field serialize.
 * @param num - The number.
 * @param bufferSize - The buffer size.
 * @returns The buffer.
 */
export function fr(n: number) {
  return new Fr(BigInt(n));
}

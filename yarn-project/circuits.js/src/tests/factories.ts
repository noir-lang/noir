import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fq, Fr } from '@aztec/foundation/fields';

import { computeCallStackItemHash } from '../abis/abis.js';
import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CallContext,
  CircuitsWasm,
  CombinedAccumulatedData,
  CombinedConstantData,
  CombinedHistoricTreeRoots,
  ConstantBaseRollupData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  KernelCircuitPublicInputs,
  MergeRollupInputs,
  NewContractData,
  NullifierLeafPreimage,
  OptionallyRevealedData,
  PreviousKernelData,
  PreviousRollupData,
  PrivateCallData,
  PrivateCircuitPublicInputs,
  PrivateHistoricTreeRoots,
  PrivateKernelInputs,
  PublicCallData,
  PublicCircuitPublicInputs,
  PublicDataRead,
  PublicDataUpdateRequest,
  PublicKernelInputs,
  PublicKernelInputsNoPreviousKernel,
  RootRollupInputs,
  RootRollupPublicInputs,
  WitnessedPublicCallData,
  PublicCallRequest,
} from '../index.js';
import { AggregationObject } from '../structs/aggregation_object.js';
import { PrivateCallStackItem, PublicCallStackItem } from '../structs/call_stack_item.js';
import {
  ARGS_LENGTH,
  CONTRACT_TREE_HEIGHT,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  EMITTED_EVENTS_LENGTH,
  FUNCTION_TREE_HEIGHT,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_L2_TO_L1_MSGS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_DATA_READS_LENGTH,
  KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
  L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT,
  L1_TO_L2_MESSAGES_SIBLING_PATH_LENGTH,
  NEW_COMMITMENTS_LENGTH,
  NEW_L2_TO_L1_MSGS_LENGTH,
  NEW_NULLIFIERS_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  PRIVATE_CALL_STACK_LENGTH,
  PRIVATE_DATA_TREE_HEIGHT,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
  PUBLIC_CALL_STACK_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  RETURN_VALUES_LENGTH,
  ROLLUP_VK_TREE_HEIGHT,
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

/**
 * Creates an arbitrary tx context with the given seed.
 * @param seed - The seed to use for generating the tx context.
 * @returns A tx context.
 */
export function makeTxContext(seed: number): TxContext {
  const deploymentData = new ContractDeploymentData(fr(seed), fr(seed + 1), fr(seed + 2), makeEthAddress(seed + 3));
  return new TxContext(false, false, true, deploymentData);
}

/**
 * Creates an arbitrary private historic tree roots object with the given seed.
 * @param seed - The seed to use for generating the private historic tree roots.
 * @returns A private historic tree roots object.
 */
export function makePrivateHistoricTreeRoots(seed: number): PrivateHistoricTreeRoots {
  return new PrivateHistoricTreeRoots(fr(seed), fr(seed + 1), fr(seed + 2), fr(seed + 3), fr(seed + 4));
}

/**
 * Creates an arbitrary combined historic tree roots object from the given seed.
 * Note: "Combined" indicates that it's the combined output of both private and public circuit flows.
 * @param seed - The seed to use for generating the combined historic tree roots.
 * @returns A combined historic tree roots object.
 */
export function makeCombinedHistoricTreeRoots(seed: number): CombinedHistoricTreeRoots {
  return new CombinedHistoricTreeRoots(makePrivateHistoricTreeRoots(seed));
}

/**
 * Creates arbitrary constant data with the given seed.
 * @param seed - The seed to use for generating the constant data.
 * @returns A constant data object.
 */
export function makeConstantData(seed = 1): CombinedConstantData {
  return new CombinedConstantData(makeCombinedHistoricTreeRoots(seed), makeTxContext(seed + 4));
}

/**
 * Creates arbitrary selector from the given seed.
 * @param seed - The seed to use for generating the selector.
 * @returns A selector.
 */
export function makeSelector(seed: number): Buffer {
  const buffer = Buffer.alloc(4);
  buffer.writeUInt32BE(seed, 0);
  return buffer;
}

/**
 * Creates arbitrary public data update request.
 * @param seed - The seed to use for generating the public data update request.
 * @returns A public data update request.
 */
export function makePublicDataUpdateRequest(seed = 1): PublicDataUpdateRequest {
  return new PublicDataUpdateRequest(fr(seed), fr(seed + 1), fr(seed + 2));
}

/**
 * Creates empty public data update request.
 * @returns An empty public data update request.
 */
export function makeEmptyPublicDataUpdateRequest(): PublicDataUpdateRequest {
  return new PublicDataUpdateRequest(fr(0), fr(0), fr(0));
}

/**
 * Creates arbitrary public data read.
 * @param seed - The seed to use for generating the public data read.
 * @returns A public data read.
 */
export function makePublicDataRead(seed = 1): PublicDataRead {
  return new PublicDataRead(fr(seed), fr(seed + 1));
}

/**
 * Creates empty public data read.
 * @returns An empty public data read.
 */
export function makeEmptyPublicDataRead(): PublicDataRead {
  return new PublicDataRead(fr(0), fr(0));
}

/**
 * Creates arbitrary contract storage update request.
 * @param seed - The seed to use for generating the contract storage update request.
 * @returns A contract storage update request.
 */
export function makeContractStorageUpdateRequest(seed = 1): ContractStorageUpdateRequest {
  return new ContractStorageUpdateRequest(fr(seed), fr(seed + 1), fr(seed + 2));
}

/**
 * Creates arbitrary contract storage read.
 * @param seed - The seed to use for generating the contract storage read.
 * @returns A contract storage read.
 */
export function makeContractStorageRead(seed = 1): ContractStorageRead {
  return new ContractStorageRead(fr(seed), fr(seed + 1));
}

/**
 * Creates empty accumulated data.
 * @param seed - The seed to use for generating the accumulated data.
 * @returns An empty accumulated data.
 */
export function makeEmptyAccumulatedData(seed = 1): CombinedAccumulatedData {
  return new CombinedAccumulatedData(
    makeAggregationObject(seed),
    range(KERNEL_NEW_COMMITMENTS_LENGTH, seed + 0x100).map(fr),
    range(KERNEL_NEW_NULLIFIERS_LENGTH, seed + 0x200).map(fr),
    new Array(KERNEL_PRIVATE_CALL_STACK_LENGTH).fill(Fr.ZERO), // private call stack must be empty
    range(KERNEL_PUBLIC_CALL_STACK_LENGTH, seed + 0x400).map(fr),
    range(KERNEL_NEW_L2_TO_L1_MSGS_LENGTH, seed + 0x500).map(fr),
    range(KERNEL_NEW_CONTRACTS_LENGTH, seed + 0x600).map(makeNewContractData),
    range(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, seed + 0x700).map(makeOptionallyRevealedData),
    range(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH, seed + 0x800).map(makeEmptyPublicDataUpdateRequest),
    range(KERNEL_PUBLIC_DATA_READS_LENGTH, seed + 0x900).map(makeEmptyPublicDataRead),
  );
}

/**
 * Creates arbitrary accumulated data.
 * @param seed - The seed to use for generating the accumulated data.
 * @returns An accumulated data.
 */
export function makeAccumulatedData(seed = 1): CombinedAccumulatedData {
  return new CombinedAccumulatedData(
    makeAggregationObject(seed),
    range(KERNEL_NEW_COMMITMENTS_LENGTH, seed + 0x100).map(fr),
    range(KERNEL_NEW_NULLIFIERS_LENGTH, seed + 0x200).map(fr),
    range(KERNEL_PRIVATE_CALL_STACK_LENGTH, seed + 0x300).map(fr),
    range(KERNEL_PUBLIC_CALL_STACK_LENGTH, seed + 0x400).map(fr),
    range(KERNEL_NEW_L2_TO_L1_MSGS_LENGTH, seed + 0x500).map(fr),
    range(KERNEL_NEW_CONTRACTS_LENGTH, seed + 0x600).map(makeNewContractData),
    range(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, seed + 0x700).map(makeOptionallyRevealedData),
    range(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH, seed + 0x800).map(makePublicDataUpdateRequest),
    range(KERNEL_PUBLIC_DATA_READS_LENGTH, seed + 0x900).map(makePublicDataRead),
  );
}

/**
 * Creates arbitrary contract data.
 * @param seed - The seed to use for generating the contract data.
 * @returns A contract data.
 */
export function makeNewContractData(seed = 1): NewContractData {
  return new NewContractData(makeAztecAddress(seed), makeEthAddress(seed + 1), fr(seed + 2));
}

/**
 * Creates arbitrary optionally revealed data.
 * @param seed - The seed to use for generating the optionally revealed data.
 * @returns An optionally revealed data.
 */
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

/**
 * Creates arbitrary aggregation object.
 * @param seed - The seed to use for generating the aggregation object.
 * @returns An aggregation object.
 */
export function makeAggregationObject(seed = 1): AggregationObject {
  return new AggregationObject(
    new AffineElement(new Fq(BigInt(seed)), new Fq(BigInt(seed + 1))),
    new AffineElement(new Fq(BigInt(seed + 0x100)), new Fq(BigInt(seed + 0x101))),
    range(4, seed + 2).map(fr),
    range(6, seed + 6),
  );
}

/**
 * Creates arbitrary call context.
 * @param seed - The seed to use for generating the call context.
 * @param storageContractAddress - The storage contract address set on the call context.
 * @returns A call context.
 */
export function makeCallContext(seed = 0, storageContractAddress = makeAztecAddress(seed + 1)): CallContext {
  return new CallContext(makeAztecAddress(seed), storageContractAddress, makeEthAddress(seed + 2), false, false, false);
}

/**
 * Creates arbitrary public circuit public inputs.
 * @param seed - The seed to use for generating the public circuit public inputs.
 * @param storageContractAddress - The storage contract address set on the call context.
 * @returns Public circuit public inputs.
 */
export function makePublicCircuitPublicInputs(
  seed = 0,
  storageContractAddress?: AztecAddress,
): PublicCircuitPublicInputs {
  const frArray = (num: number, seed: number) => range(num, seed).map(fr);
  return new PublicCircuitPublicInputs(
    makeCallContext(seed, storageContractAddress),
    frArray(ARGS_LENGTH, seed + 0x100),
    frArray(RETURN_VALUES_LENGTH, seed + 0x200),
    frArray(EMITTED_EVENTS_LENGTH, seed + 0x300),
    range(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH, seed + 0x400).map(makeContractStorageUpdateRequest),
    range(KERNEL_PUBLIC_DATA_READS_LENGTH, seed + 0x500).map(makeContractStorageRead),
    frArray(PUBLIC_CALL_STACK_LENGTH, seed + 0x600),
    frArray(NEW_L2_TO_L1_MSGS_LENGTH, seed + 0x700),
    fr(seed + 0x800),
    makeAztecAddress(seed + 0x801),
  );
}

/**
 * Creates empty kernel circuit public inputs.
 * @param seed - The seed to use for generating the kernel circuit public inputs.
 * @returns Empty kernel circuit public inputs.
 */
export function makeEmptyKernelPublicInputs(seed = 1): KernelCircuitPublicInputs {
  return new KernelCircuitPublicInputs(makeEmptyAccumulatedData(seed), makeConstantData(seed + 0x100), true);
}

/**
 * Creates arbitrary kernel circuit public inputs.
 * @param seed - The seed to use for generating the kernel circuit public inputs.
 * @returns Kernel circuit public inputs.
 */
export function makeKernelPublicInputs(seed = 1): KernelCircuitPublicInputs {
  return new KernelCircuitPublicInputs(makeAccumulatedData(seed), makeConstantData(seed + 0x100), true);
}

/**
 * Creates a public call request for testing.
 * @param seed - The seed.
 * @returns Public call request.
 */
export function makePublicCallRequest(seed = 1): PublicCallRequest {
  return new PublicCallRequest(
    makeAztecAddress(seed),
    new FunctionData(makeSelector(seed + 0x1), false, false),
    makeCallContext(seed + 0x2),
    range(ARGS_LENGTH, seed + 0x10).map(fr),
  );
}

/**
 * Creates a uint8 vector of a given size filled with a given value.
 * @param size - The size of the vector.
 * @param fill - The value to fill the vector with.
 * @returns A uint8 vector.
 */
export function makeDynamicSizeBuffer(size: number, fill: number) {
  return new UInt8Vector(Buffer.alloc(size, fill));
}

/**
 * Creates arbitrary/mocked membership witness where the sibling paths is an array of fields in an ascending order starting from `start`.
 * @param size - The size of the membership witness.
 * @param start - The start of the membership witness.
 * @returns A membership witness.
 */
export function makeMembershipWitness<N extends number>(size: number, start: number): MembershipWitness<N> {
  return new MembershipWitness(size, BigInt(start), range(size, start).map(fr));
}

/**
 * Creates arbitrary/mocked verification key.
 * @returns A verification key.
 */
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

/**
 * Makes arbitrary previous kernel data.
 * @param seed - The seed to use for generating the previous kernel data.
 * @param kernelPublicInputs - The kernel public inputs to use for generating the previous kernel data.
 * @returns A previous kernel data.
 */
export function makePreviousKernelData(seed = 1, kernelPublicInputs?: KernelCircuitPublicInputs): PreviousKernelData {
  return new PreviousKernelData(
    kernelPublicInputs ?? makeKernelPublicInputs(seed),
    makeProof(seed + 0x80),
    makeVerificationKey(),
    0x42,
    range(VK_TREE_HEIGHT, 0x1000).map(fr),
  );
}

/**
 * Makes arbitrary proof.
 * @param seed - The seed to use for generating/mocking the proof.
 * @returns A proof.
 */
export function makeProof(seed = 1) {
  return makeDynamicSizeBuffer(16, seed);
}

/**
 * Makes arbitrary private kernel inputs.
 * @param seed - The seed to use for generating the private kernel inputs.
 * @returns Private kernel inputs.
 */
export function makePrivateKernelInputs(seed = 1): PrivateKernelInputs {
  return new PrivateKernelInputs(
    makeSignedTxRequest(seed),
    makePreviousKernelData(seed + 0x1000),
    makePrivateCallData(seed + 0x2000),
  );
}

/**
 * Makes arbitrary public call stack item.
 * @param seed - The seed to use for generating the public call stack item.
 * @returns A public call stack item.
 */
export function makePublicCallStackItem(seed = 1): PublicCallStackItem {
  const callStackItem = new PublicCallStackItem(
    makeAztecAddress(seed),
    // in the public kernel, function can't be a constructor or private
    new FunctionData(makeSelector(seed + 0x1), false, false),
    makePublicCircuitPublicInputs(seed + 0x10),
    false,
  );
  callStackItem.publicInputs.callContext.storageContractAddress = callStackItem.contractAddress;
  return callStackItem;
}

/**
 * Makes arbitrary public call data.
 * @param seed - The seed to use for generating the public call data.
 * @returns A public call data.
 */
export async function makePublicCallData(seed = 1): Promise<PublicCallData> {
  const publicCallData = new PublicCallData(
    makePublicCallStackItem(seed),
    range(PUBLIC_CALL_STACK_LENGTH, seed + 0x300).map(makePublicCallStackItem),
    makeProof(seed + 0x1000),
    fr(seed + 1),
    fr(seed + 2),
  );
  // publicCallStack should be a hash of the preimages:
  const wasm = await CircuitsWasm.get();
  publicCallData.callStackItem.publicInputs.publicCallStack = [];
  publicCallData.publicCallStackPreimages.forEach(preimage => {
    publicCallData.callStackItem.publicInputs.publicCallStack.push(computeCallStackItemHash(wasm!, preimage));
  });

  // one kernel circuit call can have several methods in call stack. But all of them should have the same msg.sender - set these correctly in the preimages!
  for (let i = 0; i < publicCallData.publicCallStackPreimages.length; i++) {
    const isDelegateCall = publicCallData.publicCallStackPreimages[i].publicInputs.callContext.isDelegateCall;
    publicCallData.publicCallStackPreimages[i].publicInputs.callContext.msgSender = isDelegateCall
      ? publicCallData.callStackItem.publicInputs.callContext.msgSender
      : publicCallData.callStackItem.contractAddress;
  }

  // set the storage address for each call on the stack (handle delegatecall case)
  for (let i = 0; i < publicCallData.publicCallStackPreimages.length; i++) {
    const isDelegateCall = publicCallData.publicCallStackPreimages[i].publicInputs.callContext.isDelegateCall;
    publicCallData.publicCallStackPreimages[i].publicInputs.callContext.storageContractAddress = isDelegateCall
      ? publicCallData.callStackItem.publicInputs.callContext.storageContractAddress
      : publicCallData.publicCallStackPreimages[i].contractAddress;
  }

  return publicCallData;
}

/**
 * Makes arbitrary witnessed public call data.
 * @param seed - The seed to use for generating the witnessed public call data.
 * @returns A witnessed public call data.
 */
export async function makeWitnessedPublicCallData(seed = 1): Promise<WitnessedPublicCallData> {
  return new WitnessedPublicCallData(
    await makePublicCallData(seed),
    range(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH, seed + 0x100).map(x =>
      makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, x),
    ),
    range(KERNEL_PUBLIC_DATA_READS_LENGTH, seed + 0x200).map(x => makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, x)),
    fr(seed + 0x300),
  );
}

/**
 * Makes arbitrary public kernel inputs.
 * @param seed - The seed to use for generating the public kernel inputs.
 * @returns Public kernel inputs.
 */
export async function makePublicKernelInputs(seed = 1): Promise<PublicKernelInputs> {
  return new PublicKernelInputs(makePreviousKernelData(seed), await makePublicCallData(seed + 0x1000));
}

/**
 * Makes arbitrary public kernel inputs with empty output.
 * @param seed - The seed to use for generating the public kernel inputs.
 * @returns Public kernel inputs.
 */
export async function makePublicKernelInputsWithEmptyOutput(seed = 1): Promise<PublicKernelInputs> {
  const kernelCircuitPublicInputs = makeEmptyKernelPublicInputs(seed);
  const publicKernelInputs = new PublicKernelInputs(
    makePreviousKernelData(seed, kernelCircuitPublicInputs),
    await makePublicCallData(seed + 0x1000),
  );
  //Set the call stack item for this circuit iteration at the top of the call stack
  const wasm = await CircuitsWasm.get();
  publicKernelInputs.previousKernel.publicInputs.end.publicCallStack[KERNEL_PUBLIC_CALL_STACK_LENGTH - 1] =
    computeCallStackItemHash(wasm, publicKernelInputs.publicCallData.callStackItem);
  return publicKernelInputs;
}

/**
 * Makes arbitrary public kernel inputs with no previous kernel data.
 * @param seed - The seed to use for generating the public kernel inputs.
 * @returns Public kernel inputs.
 */
export async function makePublicKernelInputsNoKernelInput(seed = 1): Promise<PublicKernelInputsNoPreviousKernel> {
  return new PublicKernelInputsNoPreviousKernel(
    makeSignedTxRequest(seed),
    await makePublicCallData(seed + 0x100),
    makeCombinedHistoricTreeRoots(seed + 0x200),
  );
}

/**
 * Makes arbitrary signed tx request.
 * @param seed - The seed to use for generating the signed tx request.
 * @returns A signed tx request.
 */
export function makeSignedTxRequest(seed = 1): SignedTxRequest {
  return new SignedTxRequest(makeTxRequest(seed), makeEcdsaSignature(seed + 0x200));
}

/**
 * Makes arbitrary tx request.
 * @param seed - The seed to use for generating the tx request.
 * @returns A tx request.
 */
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

/**
 * Makes arbitrary private call data.
 * @param seed - The seed to use for generating the private call data.
 * @returns A private call data.
 */
export function makePrivateCallData(seed = 1): PrivateCallData {
  return PrivateCallData.from({
    callStackItem: makePrivateCallStackItem(seed),
    privateCallStackPreimages: range(PRIVATE_CALL_STACK_LENGTH, seed + 0x10).map(makePrivateCallStackItem),
    proof: makeDynamicSizeBuffer(16, seed + 0x50),
    vk: makeVerificationKey(),
    functionLeafMembershipWitness: makeMembershipWitness(FUNCTION_TREE_HEIGHT, seed + 0x30),
    contractLeafMembershipWitness: makeMembershipWitness(CONTRACT_TREE_HEIGHT, seed + 0x20),
    portalContractAddress: makeEthAddress(seed + 0x40),
    acirHash: fr(seed + 0x60),
  });
}

/**
 * Makes arbitrary private call stack item.
 * @param seed - The seed to use for generating the private call stack item.
 * @returns A private call stack item.
 */
export function makePrivateCallStackItem(seed = 1): PrivateCallStackItem {
  return new PrivateCallStackItem(
    makeAztecAddress(seed),
    new FunctionData(makeSelector(seed + 0x1), true, true),
    makePrivateCircuitPublicInputs(seed + 0x10),
  );
}

/**
 * Makes arbitrary private circuit public inputs.
 * @param seed - The seed to use for generating the private circuit public inputs.
 * @returns A private circuit public inputs.
 */
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
    newL2ToL1Msgs: range(NEW_L2_TO_L1_MSGS_LENGTH, seed + 0x800).map(fr),
    historicContractTreeRoot: fr(seed + 0x900), // TODO not in spec
    historicPrivateDataTreeRoot: fr(seed + 0x1000),
    historicPrivateNullifierTreeRoot: fr(seed + 0x1100), // TODO not in spec
    historicL1ToL2MessagesTreeRoot: fr(seed + 0x1200),
    contractDeploymentData: makeContractDeploymentData(),
  });
}

/**
 * Makes arbitrary contract deployment data.
 * @param seed - The seed to use for generating the contract deployment data.
 * @returns A contract deployment data.
 */
export function makeContractDeploymentData(seed = 1) {
  return new ContractDeploymentData(fr(seed), fr(seed + 1), fr(seed + 2), new EthAddress(numToUInt32BE(seed + 3, 20)));
}

/**
 * Makes constant base rollup data.
 * @param seed - The seed to use for generating the constant base rollup data.
 * @returns A constant base rollup data.
 */
export function makeConstantBaseRollupData(seed = 1): ConstantBaseRollupData {
  return ConstantBaseRollupData.from({
    startTreeOfHistoricPrivateDataTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed),
    startTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x100),
    startTreeOfHistoricL1ToL2MsgTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(seed + 0x200),
    privateKernelVkTreeRoot: fr(seed + 0x301),
    publicKernelVkTreeRoot: fr(seed + 0x302),
    baseRollupVkHash: fr(seed + 0x303),
    mergeRollupVkHash: fr(seed + 0x304),
  });
}

/**
 * Makes arbitrary append only tree snapshot.
 * @param seed - The seed to use for generating the append only tree snapshot.
 * @returns An append only tree snapshot.
 */
export function makeAppendOnlyTreeSnapshot(seed = 1): AppendOnlyTreeSnapshot {
  return new AppendOnlyTreeSnapshot(fr(seed), seed);
}

/**
 * Makes arbitrary eth address.
 * @param seed - The seed to use for generating the eth address.
 * @returns An eth address.
 */
export function makeEthAddress(seed = 1): EthAddress {
  return new EthAddress(Buffer.alloc(20, seed));
}

/**
 * Creates a buffer of a given size filled with a given value.
 * @param size - The size of the buffer to create.
 * @param fill - The value to fill the buffer with.
 * @returns A buffer of a given size filled with a given value.
 */
export function makeBytes(size = 32, fill = 1): Buffer {
  return Buffer.alloc(size, fill);
}

/**
 * Makes arbitrary aztec address.
 * @param seed - The seed to use for generating the aztec address.
 * @returns An aztec address.
 */
export function makeAztecAddress(seed = 1): AztecAddress {
  return new AztecAddress(fr(seed).toBuffer());
}

/**
 * Makes arbitrary ecdsa signature.
 * @param seed - The seed to use for generating the ecdsa signature.
 * @returns An ecdsa signature.
 */
export function makeEcdsaSignature(seed = 1): EcdsaSignature {
  return new EcdsaSignature(Buffer.alloc(32, seed), Buffer.alloc(32, seed + 1), Buffer.alloc(1, seed + 2));
}

/**
 * Makes arbitrary base or merge rollup circuit public inputs.
 * @param seed - The seed to use for generating the base rollup circuit public inputs.
 * @returns A base or merge rollup circuit public inputs.
 */
export function makeBaseOrMergeRollupPublicInputs(seed = 0): BaseOrMergeRollupPublicInputs {
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
    fr(seed + 0x900),
    fr(seed + 0x1000),
    [fr(seed + 0x901), fr(seed + 0x902)],
  );
}

/**
 * Makes arbitrary previous rollup data.
 * @param seed - The seed to use for generating the previous rollup data.
 * @returns A previous rollup data.
 */
export function makePreviousRollupData(seed = 0): PreviousRollupData {
  return new PreviousRollupData(
    makeBaseOrMergeRollupPublicInputs(seed),
    makeDynamicSizeBuffer(16, seed + 0x50),
    makeVerificationKey(),
    seed + 0x110,
    makeMembershipWitness(ROLLUP_VK_TREE_HEIGHT, seed + 0x120),
  );
}

/**
 * Makes root rollup inputs.
 * @param seed - The seed to use for generating the root rollup inputs.
 * @returns A root rollup inputs.
 */
export function makeRootRollupInputs(seed = 0): RootRollupInputs {
  return new RootRollupInputs(
    [makePreviousRollupData(seed), makePreviousRollupData(seed + 0x1000)],
    range(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT, 0x2000).map(fr),
    range(CONTRACT_TREE_ROOTS_TREE_HEIGHT, 0x2100).map(fr),
    range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 0x2100).map(fr),
    range(L1_TO_L2_MESSAGES_SIBLING_PATH_LENGTH, 0x2100).map(fr),
    range(L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT, 0x2100).map(fr),
    makeAppendOnlyTreeSnapshot(seed + 0x2200),
    makeAppendOnlyTreeSnapshot(seed + 0x2300),
  );
}

/**
 * Makes root rollup public inputs.
 * @param seed - The seed to use for generating the root rollup public inputs.
 * @returns A root rollup public inputs.
 */
export function makeRootRollupPublicInputs(seed = 0): RootRollupPublicInputs {
  return RootRollupPublicInputs.from({
    endAggregationObject: makeAggregationObject(seed),
    startPrivateDataTreeSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    endPrivateDataTreeSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    startNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    endNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    startContractTreeSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    endContractTreeSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    startPublicDataTreeRoot: fr((seed += 0x100)),
    endPublicDataTreeRoot: fr((seed += 0x100)),
    startTreeOfHistoricPrivateDataTreeRootsSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    endTreeOfHistoricPrivateDataTreeRootsSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    startTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    endTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    startL1ToL2MessageTreeSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    endL1ToL2MessageTreeSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: makeAppendOnlyTreeSnapshot((seed += 0x100)),
    calldataHash: [new Fr(1n), new Fr(2n)],
    l1ToL2MessagesHash: [new Fr(3n), new Fr(4n)],
  });
}

/**
 * Makes arbitrary merge rollup inputs.
 * @param seed - The seed to use for generating the merge rollup inputs.
 * @returns A merge rollup inputs.
 */
export function makeMergeRollupInputs(seed = 0): MergeRollupInputs {
  return new MergeRollupInputs([makePreviousRollupData(seed), makePreviousRollupData(seed + 0x1000)]);
}

/**
 * Makes arbitrary base rollup inputs.
 * @param seed - The seed to use for generating the base rollup inputs.
 * @returns A base rollup inputs.
 */
export function makeBaseRollupInputs(seed = 0): BaseRollupInputs {
  const kernelData: [PreviousKernelData, PreviousKernelData] = [
    makePreviousKernelData(seed + 0x100),
    makePreviousKernelData(seed + 0x200),
  ];

  const startPrivateDataTreeSnapshot = makeAppendOnlyTreeSnapshot(seed + 0x100);
  const startNullifierTreeSnapshot = makeAppendOnlyTreeSnapshot(seed + 0x200);
  const startContractTreeSnapshot = makeAppendOnlyTreeSnapshot(seed + 0x300);
  const startPublicDataTreeRoot = fr(seed + 0x400);

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

  const newPublicDataUpdateRequestsSiblingPaths = range(
    2 * KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
    seed + 0x6000,
  ).map(x => makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, x));

  const newPublicDataReadsSiblingPaths = range(2 * KERNEL_PUBLIC_DATA_READS_LENGTH, seed + 0x6000).map(x =>
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
  const historicL1ToL2MsgTreeRootMembershipWitnesses: BaseRollupInputs['historicL1ToL2MsgTreeRootMembershipWitnesses'] =
    [
      makeMembershipWitness(L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT, seed + 0xa000),
      makeMembershipWitness(L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT, seed + 0xb000),
    ];

  const constants = makeConstantBaseRollupData(0x100);

  return BaseRollupInputs.from({
    kernelData,
    startPrivateDataTreeSnapshot,
    startNullifierTreeSnapshot,
    startContractTreeSnapshot,
    startPublicDataTreeRoot,
    lowNullifierLeafPreimages,
    lowNullifierMembershipWitness,
    newCommitmentsSubtreeSiblingPath,
    newNullifiersSubtreeSiblingPath,
    newContractsSubtreeSiblingPath,
    newPublicDataUpdateRequestsSiblingPaths,
    newPublicDataReadsSiblingPaths,
    historicPrivateDataTreeRootMembershipWitnesses,
    historicContractsTreeRootMembershipWitnesses,
    historicL1ToL2MsgTreeRootMembershipWitnesses,
    constants,
  });
}

/**
 * TODO: Since the max value check is currently disabled this function is pointless. Should it be removed?
 * Test only. Easy to identify big endian field serialize.
 * @param n - The number.
 * @returns The field.
 */
export function fr(n: number): Fr {
  return new Fr(BigInt(n));
}

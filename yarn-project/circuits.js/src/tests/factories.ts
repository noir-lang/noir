import { makeHalfFullTuple, makeTuple, range } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { numToUInt32BE } from '@aztec/foundation/serialize';

import { randomBytes } from 'crypto';

import { SchnorrSignature } from '../barretenberg/index.js';
import {
  ARCHIVE_HEIGHT,
  ARGS_LENGTH,
  AggregationObject,
  AppendOnlyTreeSnapshot,
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  BlockHeader,
  CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
  CONTRACT_TREE_HEIGHT,
  CallContext,
  CallRequest,
  CallerContext,
  CircuitType,
  CombinedAccumulatedData,
  CombinedConstantData,
  ConstantRollupData,
  ContractDeploymentData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  FUNCTION_TREE_HEIGHT,
  FinalAccumulatedData,
  Fq,
  Fr,
  FunctionData,
  FunctionSelector,
  G1AffineElement,
  GrumpkinPrivateKey,
  GrumpkinScalar,
  KernelCircuitPublicInputs,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_READ_REQUESTS_PER_CALL,
  MAX_READ_REQUESTS_PER_TX,
  MembershipWitness,
  MergeRollupInputs,
  NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
  NOTE_HASH_TREE_HEIGHT,
  NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_FIELDS_PER_SHA256,
  NewContractData,
  NullifierKeyValidationRequest,
  NullifierKeyValidationRequestContext,
  NullifierLeafPreimage,
  OptionallyRevealedData,
  PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  PartialStateReference,
  Point,
  PreviousKernelData,
  PreviousRollupData,
  PrivateCallData,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelPublicInputsFinal,
  Proof,
  PublicCallData,
  PublicCallRequest,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicDataRead,
  PublicDataTreeLeaf,
  PublicDataTreeLeafPreimage,
  PublicDataUpdateRequest,
  PublicKernelInputs,
  RETURN_VALUES_LENGTH,
  ROLLUP_VK_TREE_HEIGHT,
  ReadRequestMembershipWitness,
  RollupTypes,
  RootRollupInputs,
  RootRollupPublicInputs,
  SideEffect,
  SideEffectLinkedToNoteHash,
  StateDiffHints,
  StateReference,
  TxContext,
  TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  WitnessedPublicCallData,
} from '../index.js';
import { GlobalVariables } from '../structs/global_variables.js';
import { Header, NUM_BYTES_PER_SHA256 } from '../structs/header.js';

/**
 * Creates an arbitrary side effect object with the given seed.
 * @param seed - The seed to use for generating the object.
 * @returns A side effect object.
 */
export function makeNewSideEffect(seed: number): SideEffect {
  return new SideEffect(fr(seed), fr(seed + 1));
}

/**
 * Creates an arbitrary side effect object (linked to a note hash) with the given seed.
 * @param seed - The seed to use for generating the object.
 * @returns A side effect object.
 */
export function makeNewSideEffectLinkedToNoteHash(seed: number): SideEffectLinkedToNoteHash {
  return new SideEffectLinkedToNoteHash(fr(seed), fr(seed + 1), fr(seed + 2));
}

/**
 * Creates an arbitrary tx context with the given seed.
 * @param seed - The seed to use for generating the tx context.
 * @returns A tx context.
 */
export function makeTxContext(seed: number): TxContext {
  // @todo @LHerskind should probably take value for chainId as it will be verified later.
  // @todo @LHerskind should probably take value for version as it will be verified later.
  return new TxContext(false, false, true, makeContractDeploymentData(seed), Fr.ZERO, Fr.ZERO);
}

/**
 * Creates an arbitrary combined historical tree roots object from the given seed.
 * Note: "Combined" indicates that it's the combined output of both private and public circuit flows.
 * @param seed - The seed to use for generating the combined historical tree roots.
 * @returns A combined historical tree roots object.
 */
export function makeBlockHeader(seed: number): BlockHeader {
  return new BlockHeader(
    fr(seed),
    fr(seed + 1),
    fr(seed + 2),
    fr(seed + 3),
    fr(seed + 4),
    fr(seed + 5),
    fr(seed + 6),
    fr(seed + 7),
  );
}

/**
 * Creates arbitrary constant data with the given seed.
 * @param seed - The seed to use for generating the constant data.
 * @returns A constant data object.
 */
export function makeConstantData(seed = 1): CombinedConstantData {
  return new CombinedConstantData(makeBlockHeader(seed), makeTxContext(seed + 4));
}

/**
 * Creates arbitrary selector from the given seed.
 * @param seed - The seed to use for generating the selector.
 * @returns A selector.
 */
export function makeSelector(seed: number): FunctionSelector {
  return new FunctionSelector(seed);
}

/**
 * Creates arbitrary NullifierKeyValidationRequest from the given seed.
 * @param seed - The seed to use for generating the NullifierKeyValidationRequest.
 * @returns A NullifierKeyValidationRequest.
 */
function makeNullifierKeyValidationRequest(seed: number): NullifierKeyValidationRequest {
  return new NullifierKeyValidationRequest(makePoint(seed), makeGrumpkinPrivateKey(seed + 2));
}

/**
 * Creates arbitrary NullifierKeyValidationRequestContext from the given seed.
 * @param seed - The seed to use for generating the NullifierKeyValidationRequestContext.
 * @returns A NullifierKeyValidationRequestContext.
 */
function makeNullifierKeyValidationRequestContext(seed: number): NullifierKeyValidationRequestContext {
  return new NullifierKeyValidationRequestContext(
    makePoint(seed),
    makeGrumpkinPrivateKey(seed + 2),
    makeAztecAddress(seed + 4),
  );
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
 * Creates arbitrary accumulated data.
 * @param seed - The seed to use for generating the accumulated data.
 * @returns An accumulated data.
 */
export function makeAccumulatedData(seed = 1, full = false): CombinedAccumulatedData {
  const tupleGenerator = full ? makeTuple : makeHalfFullTuple;

  return new CombinedAccumulatedData(
    makeAggregationObject(seed),
    tupleGenerator(MAX_READ_REQUESTS_PER_TX, sideEffectFromNumber, seed + 0x80),
    tupleGenerator(
      MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
      makeNullifierKeyValidationRequestContext,
      seed + 0x100,
    ),
    tupleGenerator(MAX_NEW_COMMITMENTS_PER_TX, sideEffectFromNumber, seed + 0x120),
    tupleGenerator(MAX_NEW_NULLIFIERS_PER_TX, sideEffectLinkedFromNumber, seed + 0x200),
    tupleGenerator(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x400),
    tupleGenerator(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x500),
    tupleGenerator(MAX_NEW_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x600),
    tupleGenerator(2, fr, seed + 0x700), // encrypted logs hash
    tupleGenerator(2, fr, seed + 0x800), // unencrypted logs hash
    fr(seed + 0x900), // encrypted_log_preimages_length
    fr(seed + 0xa00), // unencrypted_log_preimages_length
    tupleGenerator(MAX_NEW_CONTRACTS_PER_TX, makeNewContractData, seed + 0xb00),
    tupleGenerator(MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX, makeOptionallyRevealedData, seed + 0xc00),
    tupleGenerator(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, makePublicDataUpdateRequest, seed + 0xd00),
    tupleGenerator(MAX_PUBLIC_DATA_READS_PER_TX, makePublicDataRead, seed + 0xe00),
  );
}

/**
 * Creates arbitrary final accumulated data.
 * @param seed - The seed to use for generating the final accumulated data.
 * @returns A final accumulated data.
 */
export function makeFinalAccumulatedData(seed = 1, full = false): FinalAccumulatedData {
  const tupleGenerator = full ? makeTuple : makeHalfFullTuple;

  return new FinalAccumulatedData(
    makeAggregationObject(seed),
    tupleGenerator(MAX_NEW_COMMITMENTS_PER_TX, sideEffectFromNumber, seed + 0x100),
    tupleGenerator(MAX_NEW_NULLIFIERS_PER_TX, sideEffectLinkedFromNumber, seed + 0x200),
    tupleGenerator(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x400),
    tupleGenerator(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x500),
    tupleGenerator(MAX_NEW_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x600),
    tupleGenerator(2, fr, seed + 0x700), // encrypted logs hash
    tupleGenerator(2, fr, seed + 0x800), // unencrypted logs hash
    fr(seed + 0x900), // encrypted_log_preimages_length
    fr(seed + 0xa00), // unencrypted_log_preimages_length
    tupleGenerator(MAX_NEW_CONTRACTS_PER_TX, makeNewContractData, seed + 0xb00),
    tupleGenerator(MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX, makeOptionallyRevealedData, seed + 0xc00),
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
    new FunctionData(makeSelector(seed + 1), false, true, true),
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
    new G1AffineElement(new Fq(BigInt(seed)), new Fq(BigInt(seed + 1))),
    new G1AffineElement(new Fq(BigInt(seed + 0x100)), new Fq(BigInt(seed + 0x101))),
    makeTuple(4, fr, seed + 2),
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
  return new CallContext(
    makeAztecAddress(seed),
    storageContractAddress,
    makeEthAddress(seed + 2),
    makeSelector(seed + 3),
    false,
    false,
    false,
    0,
  );
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
  full = false,
): PublicCircuitPublicInputs {
  const tupleGenerator = full ? makeTuple : makeHalfFullTuple;

  return new PublicCircuitPublicInputs(
    makeCallContext(seed, storageContractAddress),
    fr(seed + 0x100),
    tupleGenerator(RETURN_VALUES_LENGTH, fr, seed + 0x200),
    tupleGenerator(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, makeContractStorageUpdateRequest, seed + 0x400),
    tupleGenerator(MAX_PUBLIC_DATA_READS_PER_CALL, makeContractStorageRead, seed + 0x500),
    tupleGenerator(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, fr, seed + 0x600),
    tupleGenerator(MAX_NEW_COMMITMENTS_PER_CALL, makeNewSideEffect, seed + 0x700),
    tupleGenerator(MAX_NEW_NULLIFIERS_PER_CALL, makeNewSideEffectLinkedToNoteHash, seed + 0x800),
    tupleGenerator(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, fr, seed + 0x900),
    tupleGenerator(2, fr, seed + 0x901),
    fr(seed + 0x902),
    makeBlockHeader(seed + 0xa00),
    makeAztecAddress(seed + 0xb01),
  );
}

/**
 * Creates arbitrary kernel circuit public inputs.
 * @param seed - The seed to use for generating the kernel circuit public inputs.
 * @returns Kernel circuit public inputs.
 */
export function makeKernelPublicInputs(seed = 1, fullAccumulatedData = true): KernelCircuitPublicInputs {
  return new KernelCircuitPublicInputs(
    makeAccumulatedData(seed, fullAccumulatedData),
    makeConstantData(seed + 0x100),
    true,
  );
}

/**
 * Creates arbitrary final ordering kernel circuit public inputs.
 * @param seed - The seed to use for generating the final ordering kernel circuit public inputs.
 * @returns Final ordering kernel circuit public inputs.
 */
export function makePrivateKernelPublicInputsFinal(seed = 1): PrivateKernelPublicInputsFinal {
  return new PrivateKernelPublicInputsFinal(makeFinalAccumulatedData(seed, true), makeConstantData(seed + 0x100));
}

/**
 * Creates a public call request for testing.
 * @param seed - The seed.
 * @returns Public call request.
 */
export function makePublicCallRequest(seed = 1): PublicCallRequest {
  return new PublicCallRequest(
    makeAztecAddress(seed),
    new FunctionData(makeSelector(seed + 0x1), false, false, false),
    makeCallContext(seed + 0x2, makeAztecAddress(seed)),
    makeTuple(ARGS_LENGTH, fr, seed + 0x10),
  );
}

/**
 * Creates a uint8 vector of a given size filled with a given value.
 * @param size - The size of the vector.
 * @param fill - The value to fill the vector with.
 * @returns A uint8 vector.
 */
export function makeDynamicSizeBuffer(size: number, fill: number) {
  return new Proof(Buffer.alloc(size, fill));
}

/**
 * Creates arbitrary/mocked membership witness where the sibling paths is an array of fields in an ascending order starting from `start`.
 * @param size - The size of the membership witness.
 * @param start - The start of the membership witness.
 * @returns A membership witness.
 */
export function makeMembershipWitness<N extends number>(size: N, start: number): MembershipWitness<N> {
  return new MembershipWitness(size, BigInt(start), makeTuple(size, fr, start));
}

/**
 * Creates arbitrary/mocked membership witness where the sibling paths is an array of fields in an ascending order starting from `start`.
 * @param start - The start of the membership witness.
 * @returns A non-transient read request membership witness.
 */
export function makeReadRequestMembershipWitness(start: number): ReadRequestMembershipWitness {
  return new ReadRequestMembershipWitness(
    new Fr(start),
    makeTuple(NOTE_HASH_TREE_HEIGHT, fr, start + 1),
    false,
    new Fr(0),
  );
}

/**
 * Creates empty membership witness where the sibling paths is an array of fields filled with zeros.
 * @param start - The start of the membership witness.
 * @returns Non-transient empty read request membership witness.
 */
export function makeEmptyReadRequestMembershipWitness(): ReadRequestMembershipWitness {
  return new ReadRequestMembershipWitness(new Fr(0), makeTuple(NOTE_HASH_TREE_HEIGHT, Fr.zero), false, new Fr(0));
}

/**
 * Creates arbitrary/mocked verification key.
 * @returns A verification key.
 */
export function makeVerificationKey(): VerificationKey {
  return new VerificationKey(
    CircuitType.STANDARD,
    101, // arbitrary
    102, // arbitrary
    {
      A: new G1AffineElement(new Fq(0x200), new Fq(0x300)),
    },
    /* recursive proof */ true,
    range(5, 400),
  );
}

/**
 * Creates an arbitrary point in a curve.
 * @param seed - Seed to generate the point values.
 * @returns A point.
 */
export function makePoint(seed = 1): Point {
  return new Point(fr(seed), fr(seed + 1));
}

/**
 * Creates an arbitrary grumpkin private key.
 * @param seed - Seed to generate the values.
 * @returns A GrumpkinPrivateKey.
 */
export function makeGrumpkinPrivateKey(seed = 1): GrumpkinPrivateKey {
  return GrumpkinScalar.fromHighLow(fr(seed), fr(seed + 1));
}

/**
 * Makes arbitrary previous kernel data.
 * @param seed - The seed to use for generating the previous kernel data.
 * @param kernelPublicInputs - The kernel public inputs to use for generating the previous kernel data.
 * @returns A previous kernel data.
 */
export function makePreviousKernelData(seed = 1, kernelPublicInputs?: KernelCircuitPublicInputs): PreviousKernelData {
  return new PreviousKernelData(
    kernelPublicInputs ?? makeKernelPublicInputs(seed, true),
    new Proof(Buffer.alloc(16, seed + 0x80)),
    makeVerificationKey(),
    0x42,
    makeTuple(VK_TREE_HEIGHT, fr, 0x1000),
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
 * Makes arbitrary private kernel inputs - initial call.
 * @param seed - The seed to use for generating the private kernel inputs.
 * @returns Private kernel inputs.
 */
export function makePrivateKernelInputsInit(seed = 1): PrivateKernelInputsInit {
  return new PrivateKernelInputsInit(makeTxRequest(seed), makePrivateCallData(seed + 0x1000));
}

/**
 * Makes arbitrary private kernel inputs - inner call.
 * @param seed - The seed to use for generating the private kernel inputs.
 * @returns Private kernel inputs.
 */
export function makePrivateKernelInputsInner(seed = 1): PrivateKernelInputsInner {
  return new PrivateKernelInputsInner(makePreviousKernelData(seed), makePrivateCallData(seed + 0x1000));
}

/**
 * Makes arbitrary call stack item.
 * @param seed - The seed to use for generating the call stack item.
 * @returns A call stack item.
 */
export function makeCallerContext(seed = 1): CallerContext {
  return new CallerContext(makeAztecAddress(seed), makeAztecAddress(seed + 0x1));
}

/**
 * Makes arbitrary call stack item.
 * @param seed - The seed to use for generating the call stack item.
 * @returns A call stack item.
 */
export function makeCallRequest(seed = 1): CallRequest {
  return new CallRequest(fr(seed), makeAztecAddress(seed + 0x1), makeCallerContext(seed + 0x2), fr(0), fr(0));
}

/**
 * Makes arbitrary public call stack item.
 * @param seed - The seed to use for generating the public call stack item.
 * @returns A public call stack item.
 */
export function makePublicCallStackItem(seed = 1, full = false): PublicCallStackItem {
  const callStackItem = new PublicCallStackItem(
    makeAztecAddress(seed),
    // in the public kernel, function can't be a constructor or private
    new FunctionData(makeSelector(seed + 0x1), false, false, false),
    makePublicCircuitPublicInputs(seed + 0x10, undefined, full),
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
export function makePublicCallData(seed = 1, full = false): PublicCallData {
  const publicCallData = new PublicCallData(
    makePublicCallStackItem(seed, full),
    makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, makeCallRequest, seed + 0x300),
    makeProof(),
    fr(seed + 1),
    fr(seed + 2),
  );

  return publicCallData;
}

/**
 * Makes arbitrary witnessed public call data.
 * @param seed - The seed to use for generating the witnessed public call data.
 * @returns A witnessed public call data.
 */
export function makeWitnessedPublicCallData(seed = 1): WitnessedPublicCallData {
  return new WitnessedPublicCallData(
    makePublicCallData(seed),
    range(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, seed + 0x100).map(x =>
      makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, x),
    ),
    makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, x => makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, x), seed + 0x200),
    fr(seed + 0x300),
  );
}

/**
 * Makes arbitrary public kernel inputs.
 * @param seed - The seed to use for generating the public kernel inputs.
 * @returns Public kernel inputs.
 */
export function makePublicKernelInputs(seed = 1): PublicKernelInputs {
  return new PublicKernelInputs(makePreviousKernelData(seed), makePublicCallData(seed + 0x1000));
}

/**
 * Makes arbitrary public kernel inputs.
 * @param seed - The seed to use for generating the public kernel inputs.
 * @param tweak - An optional function to tweak the output before computing hashes.
 * @returns Public kernel inputs.
 */
export function makePublicKernelInputsWithTweak(
  seed = 1,
  tweak?: (publicKernelInputs: PublicKernelInputs) => void,
): PublicKernelInputs {
  const kernelCircuitPublicInputs = makeKernelPublicInputs(seed, false);
  const previousKernel = makePreviousKernelData(seed, kernelCircuitPublicInputs);
  const publicCall = makePublicCallData(seed + 0x1000);
  const publicKernelInputs = new PublicKernelInputs(previousKernel, publicCall);
  if (tweak) {
    tweak(publicKernelInputs);
  }
  // Set the call stack item for this circuit iteration at the top of the call stack
  publicKernelInputs.previousKernel.publicInputs.end.publicCallStack[MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX - 1] =
    new CallRequest(
      publicCall.callStackItem.hash(),
      publicCall.callStackItem.publicInputs.callContext.msgSender,
      makeCallerContext(seed + 0x100),
      Fr.ZERO,
      Fr.ZERO,
    );
  return publicKernelInputs;
}

/**
 * Makes arbitrary tx request.
 * @param seed - The seed to use for generating the tx request.
 * @returns A tx request.
 */
export function makeTxRequest(seed = 1): TxRequest {
  return TxRequest.from({
    origin: makeAztecAddress(seed),
    functionData: new FunctionData(makeSelector(seed + 0x100), false, true, true),
    argsHash: fr(seed + 0x200),
    txContext: makeTxContext(seed + 0x400),
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
    privateCallStack: makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, makeCallRequest, seed + 0x10),
    publicCallStack: makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, makeCallRequest, seed + 0x20),
    proof: new Proof(Buffer.alloc(16).fill(seed + 0x50)),
    vk: makeVerificationKey(),
    functionLeafMembershipWitness: makeMembershipWitness(FUNCTION_TREE_HEIGHT, seed + 0x30),
    contractLeafMembershipWitness: makeMembershipWitness(CONTRACT_TREE_HEIGHT, seed + 0x20),
    readRequestMembershipWitnesses: makeTuple(
      MAX_READ_REQUESTS_PER_CALL,
      makeReadRequestMembershipWitness,
      seed + 0x70,
    ),
    portalContractAddress: makeEthAddress(seed + 0x40).toField(),
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
    new FunctionData(makeSelector(seed + 0x1), false, true, true),
    makePrivateCircuitPublicInputs(seed + 0x10),
    false,
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
      makeSelector(seed + 4),
      true,
      true,
      true,
      0,
    ),
    argsHash: fr(seed + 0x100),
    returnValues: makeTuple(RETURN_VALUES_LENGTH, fr, seed + 0x200),
    readRequests: makeTuple(MAX_READ_REQUESTS_PER_CALL, sideEffectFromNumber, seed + 0x300),
    nullifierKeyValidationRequests: makeTuple(
      MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL,
      makeNullifierKeyValidationRequest,
      seed + 0x300,
    ),
    newCommitments: makeTuple(MAX_NEW_COMMITMENTS_PER_CALL, sideEffectFromNumber, seed + 0x400),
    newNullifiers: makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, sideEffectLinkedFromNumber, seed + 0x500),
    privateCallStackHashes: makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, fr, seed + 0x600),
    publicCallStackHashes: makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, fr, seed + 0x700),
    newL2ToL1Msgs: makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, fr, seed + 0x800),
    endSideEffectCounter: fr(seed + 0x850),
    encryptedLogsHash: makeTuple(NUM_FIELDS_PER_SHA256, fr, seed + 0x900),
    unencryptedLogsHash: makeTuple(NUM_FIELDS_PER_SHA256, fr, seed + 0xa00),
    encryptedLogPreimagesLength: fr(seed + 0xb00),
    unencryptedLogPreimagesLength: fr(seed + 0xc00),
    blockHeader: makeBlockHeader(seed + 0xd00),
    contractDeploymentData: makeContractDeploymentData(seed + 0xe00),
    chainId: fr(seed + 0x1400),
    version: fr(seed + 0x1500),
  });
}

/**
 * Makes arbitrary contract deployment data.
 * @param seed - The seed to use for generating the contract deployment data.
 * @returns A contract deployment data.
 */
export function makeContractDeploymentData(seed = 1) {
  return new ContractDeploymentData(
    makePoint(seed),
    fr(seed + 1),
    fr(seed + 2),
    fr(seed + 3),
    makeEthAddress(seed + 4),
  );
}

/**
 * Makes global variables.
 * @param seed - The seed to use for generating the global variables.
 * @param blockNumber - The block number to use for generating the global variables.
 * If blockNumber is undefined, it will be set to seed + 2.
 * @returns Global variables.
 */
export function makeGlobalVariables(seed = 1, blockNumber: number | undefined = undefined): GlobalVariables {
  if (blockNumber !== undefined) {
    return new GlobalVariables(fr(seed), fr(seed + 1), fr(blockNumber), fr(seed + 3));
  }
  return new GlobalVariables(fr(seed), fr(seed + 1), fr(seed + 2), fr(seed + 3));
}

/**
 * Makes constant base rollup data.
 * @param seed - The seed to use for generating the constant base rollup data.
 * @param blockNumber - The block number to use for generating the global variables.
 * @returns A constant base rollup data.
 */
export function makeConstantBaseRollupData(
  seed = 1,
  globalVariables: GlobalVariables | undefined = undefined,
): ConstantRollupData {
  return ConstantRollupData.from({
    lastArchive: makeAppendOnlyTreeSnapshot(seed + 0x300),
    privateKernelVkTreeRoot: fr(seed + 0x401),
    publicKernelVkTreeRoot: fr(seed + 0x402),
    baseRollupVkHash: fr(seed + 0x403),
    mergeRollupVkHash: fr(seed + 0x404),
    globalVariables: globalVariables ?? makeGlobalVariables(seed + 0x405),
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
 * Makes arbitrary Schnorr signature.
 * @param seed - The seed to use for generating the Schnorr signature.
 * @returns A Schnorr signature.
 */
export function makeSchnorrSignature(seed = 1): SchnorrSignature {
  return new SchnorrSignature(Buffer.alloc(SchnorrSignature.SIZE, seed));
}

/**
 * Makes arbitrary base or merge rollup circuit public inputs.
 * @param seed - The seed to use for generating the base rollup circuit public inputs.
 * @param blockNumber - The block number to use for generating the base rollup circuit public inputs.
 * @returns A base or merge rollup circuit public inputs.
 */
export function makeBaseOrMergeRollupPublicInputs(
  seed = 0,
  globalVariables: GlobalVariables | undefined = undefined,
): BaseOrMergeRollupPublicInputs {
  return new BaseOrMergeRollupPublicInputs(
    RollupTypes.Base,
    new Fr(0n),
    makeAggregationObject(seed + 0x100),
    makeConstantBaseRollupData(seed + 0x200, globalVariables),
    makePartialStateReference(seed + 0x300),
    makePartialStateReference(seed + 0x400),
    [fr(seed + 0x901), fr(seed + 0x902)],
  );
}

/**
 * Makes arbitrary previous rollup data.
 * @param seed - The seed to use for generating the previous rollup data.
 * @param globalVariables - The global variables to use when generating the previous rollup data.
 * @returns A previous rollup data.
 */
export function makePreviousRollupData(
  seed = 0,
  globalVariables: GlobalVariables | undefined = undefined,
): PreviousRollupData {
  return new PreviousRollupData(
    makeBaseOrMergeRollupPublicInputs(seed, globalVariables),
    makeDynamicSizeBuffer(16, seed + 0x50),
    makeVerificationKey(),
    seed + 0x110,
    makeMembershipWitness(ROLLUP_VK_TREE_HEIGHT, seed + 0x120),
  );
}

/**
 * Makes root rollup inputs.
 * @param seed - The seed to use for generating the root rollup inputs.
 * @param blockNumber - The block number to use for generating the root rollup inputs.
 * @returns A root rollup inputs.
 */
export function makeRootRollupInputs(seed = 0, globalVariables?: GlobalVariables): RootRollupInputs {
  return new RootRollupInputs(
    [makePreviousRollupData(seed, globalVariables), makePreviousRollupData(seed + 0x1000, globalVariables)],
    makeTuple(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, fr, 0x2100),
    makeTuple(L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH, fr, 0x2100),
    makeAppendOnlyTreeSnapshot(seed + 0x2200),
    makeAppendOnlyTreeSnapshot(seed + 0x2200),
    makeTuple(ARCHIVE_HEIGHT, fr, 0x2400),
  );
}

/**
 * Makes root rollup public inputs.
 * @param seed - The seed to use for generating the root rollup public inputs.
 * @param blockNumber - The block number to use for generating the root rollup public inputs.
 * if blockNumber is undefined, it will be set to seed + 2.
 * @returns A root rollup public inputs.
 */
export function makeRootRollupPublicInputs(
  seed = 0,
  globalVariables: GlobalVariables | undefined = undefined,
): RootRollupPublicInputs {
  return RootRollupPublicInputs.from({
    aggregationObject: makeAggregationObject(seed),
    archive: makeAppendOnlyTreeSnapshot(seed + 0x100),
    header: makeHeader(seed + 0x200, globalVariables),
    l1ToL2MessagesHash: [new Fr(3n), new Fr(4n)],
  });
}

/**
 * Makes header.
 */
export function makeHeader(seed = 0, globalVariables: GlobalVariables | undefined): Header {
  return new Header(
    makeAppendOnlyTreeSnapshot(seed + 0x100),
    randomBytes(NUM_BYTES_PER_SHA256),
    makeStateReference(seed + 0x200),
    globalVariables ?? makeGlobalVariables((seed += 0x100)),
  );
}

/**
 * Makes arbitrary state reference.
 * @param seed - The seed to use for generating the state reference.
 * @returns A state reference.
 */
export function makeStateReference(seed = 0): StateReference {
  return new StateReference(makeAppendOnlyTreeSnapshot(seed), makePartialStateReference(seed + 1));
}

/**
 * Makes arbitrary partial state reference.
 * @param seed - The seed to use for generating the partial state reference.
 * @returns A partial state reference.
 */
export function makePartialStateReference(seed = 0): PartialStateReference {
  return new PartialStateReference(
    makeAppendOnlyTreeSnapshot(seed),
    makeAppendOnlyTreeSnapshot(seed + 1),
    makeAppendOnlyTreeSnapshot(seed + 2),
    makeAppendOnlyTreeSnapshot(seed + 3),
  );
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
 * Makes arbitrary public data tree leaves.
 * @param seed - The seed to use for generating the public data tree leaf.
 * @returns A public data tree leaf.
 */
export function makePublicDataTreeLeaf(seed = 0): PublicDataTreeLeaf {
  return new PublicDataTreeLeaf(new Fr(seed), new Fr(seed + 1));
}

/**
 * Makes arbitrary public data tree leaf preimages.
 * @param seed - The seed to use for generating the public data tree leaf preimage.
 * @returns A public data tree leaf preimage.
 */
export function makePublicDataTreeLeafPreimage(seed = 0): PublicDataTreeLeafPreimage {
  return new PublicDataTreeLeafPreimage(new Fr(seed), new Fr(seed + 1), new Fr(seed + 2), BigInt(seed + 3));
}

/**
 * Creates an instance of StateDiffHints with arbitrary values based on the provided seed.
 * @param seed - The seed to use for generating the hints.
 * @returns A StateDiffHints object.
 */
export function makeStateDiffHints(seed = 1): StateDiffHints {
  const nullifierPredecessorPreimages = makeTuple(
    MAX_NEW_NULLIFIERS_PER_TX,
    x => new NullifierLeafPreimage(fr(x), fr(x + 0x100), BigInt(x + 0x200)),
    seed + 0x1000,
  );

  const nullifierPredecessorMembershipWitnesses = makeTuple(
    MAX_NEW_NULLIFIERS_PER_TX,
    x => makeMembershipWitness(NULLIFIER_TREE_HEIGHT, x),
    seed + 0x2000,
  );

  const sortedNullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, fr, seed + 0x3000);

  const sortedNullifierIndexes = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, i => i, seed + 0x4000);

  const noteHashSubtreeSiblingPath = makeTuple(NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH, fr, seed + 0x5000);

  const nullifierSubtreeSiblingPath = makeTuple(NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, fr, seed + 0x6000);

  const contractSubtreeSiblingPath = makeTuple(CONTRACT_SUBTREE_SIBLING_PATH_LENGTH, fr, seed + 0x7000);

  const publicDataSiblingPath = makeTuple(PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, fr, 0x8000);

  return new StateDiffHints(
    nullifierPredecessorPreimages,
    nullifierPredecessorMembershipWitnesses,
    sortedNullifiers,
    sortedNullifierIndexes,
    noteHashSubtreeSiblingPath,
    nullifierSubtreeSiblingPath,
    contractSubtreeSiblingPath,
    publicDataSiblingPath,
  );
}

/**
 * Makes arbitrary base rollup inputs.
 * @param seed - The seed to use for generating the base rollup inputs.
 * @returns A base rollup inputs.
 */
export function makeBaseRollupInputs(seed = 0): BaseRollupInputs {
  const kernelData = makePreviousKernelData(seed);

  const start = makePartialStateReference(seed + 0x100);

  const stateDiffHints = makeStateDiffHints(seed + 0x600);

  const sortedPublicDataWrites = makeTuple(
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    makePublicDataTreeLeaf,
    seed + 0x8000,
  );

  const sortedPublicDataWritesIndexes = makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => i, 0);

  const lowPublicDataWritesPreimages = makeTuple(
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    makePublicDataTreeLeafPreimage,
    seed + 0x8200,
  );

  const lowPublicDataWritesMembershipWitnesses = makeTuple(
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    i => makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, i),
    seed + 0x8400,
  );

  const publicDataReadsPreimages = makeTuple(
    MAX_PUBLIC_DATA_READS_PER_TX,
    makePublicDataTreeLeafPreimage,
    seed + 0x8800,
  );

  const publicDataReadsMembershipWitnesses = makeTuple(
    MAX_PUBLIC_DATA_READS_PER_TX,
    i => makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, i),
    seed + 0x8a00,
  );

  const archiveRootMembershipWitness = makeMembershipWitness(ARCHIVE_HEIGHT, seed + 0x9000);

  const constants = makeConstantBaseRollupData(0x100);

  return BaseRollupInputs.from({
    kernelData,
    start,
    stateDiffHints,
    sortedPublicDataWrites,
    sortedPublicDataWritesIndexes,
    lowPublicDataWritesPreimages,
    lowPublicDataWritesMembershipWitnesses,
    publicDataReadsPreimages,
    publicDataReadsMembershipWitnesses,
    archiveRootMembershipWitness,
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

/**
 * Test only. Easy to identify big endian side-effect serialize.
 * @param n - The number.
 * @returns The SideEffect instance.
 */
export function sideEffectFromNumber(n: number): SideEffect {
  return new SideEffect(new Fr(BigInt(n)), Fr.zero());
}

/**
 * Test only. Easy to identify big endian side-effect serialize.
 * @param n - The number.
 * @returns The SideEffect instance.
 */
export function sideEffectLinkedFromNumber(n: number): SideEffectLinkedToNoteHash {
  return new SideEffectLinkedToNoteHash(new Fr(BigInt(n)), Fr.zero(), Fr.zero());
}

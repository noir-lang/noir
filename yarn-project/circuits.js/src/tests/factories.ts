import { type FieldsOf, makeHalfFullTuple, makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { EthAddress } from '@aztec/foundation/eth-address';
import { type Bufferable } from '@aztec/foundation/serialize';
import {
  type ContractClassPublic,
  type ExecutablePrivateFunctionWithMembershipProof,
  type PrivateFunction,
  type PublicFunction,
  type UnconstrainedFunctionWithMembershipProof,
} from '@aztec/types/contracts';

import { SchnorrSignature } from '../barretenberg/index.js';
import {
  ARCHIVE_HEIGHT,
  ARGS_LENGTH,
  AppendOnlyTreeSnapshot,
  AvmCircuitInputs,
  AvmExecutionHints,
  AvmExternalCallHint,
  AvmKeyValueHint,
  BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  BaseRollupInputs,
  CallContext,
  CallRequest,
  CallerContext,
  CombineHints,
  CombinedAccumulatedData,
  CombinedConstantData,
  ConstantRollupData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  EncryptedLogHash,
  Fr,
  FunctionData,
  FunctionSelector,
  type GrumpkinPrivateKey,
  GrumpkinScalar,
  KeyValidationRequest,
  KeyValidationRequestAndGenerator,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  L2ToL1Message,
  LogHash,
  MAX_ENCRYPTED_LOGS_PER_CALL,
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_KEY_VALIDATION_REQUESTS_PER_CALL,
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_CALL,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_CALL,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_HINTS,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_CALL,
  MAX_UNENCRYPTED_LOGS_PER_TX,
  MaxBlockNumber,
  MembershipWitness,
  MergeRollupInputs,
  NESTED_RECURSIVE_PROOF_LENGTH,
  NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
  NUM_MSGS_PER_BASE_PARITY,
  NoteHash,
  NoteLogHash,
  Nullifier,
  NullifierLeafPreimage,
  NullifierNonExistentReadRequestHintsBuilder,
  NullifierReadRequestHintsBuilder,
  PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  ParityPublicInputs,
  PartialPrivateTailPublicInputsForPublic,
  PartialPrivateTailPublicInputsForRollup,
  PartialStateReference,
  Point,
  PreviousRollupData,
  PrivateCallRequest,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PrivateKernelTailCircuitPublicInputs,
  Proof,
  PublicAccumulatedData,
  PublicCallData,
  PublicCallRequest,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicDataHint,
  PublicDataRead,
  PublicDataReadRequestHintsBuilder,
  PublicDataTreeLeaf,
  PublicDataTreeLeafPreimage,
  PublicDataUpdateRequest,
  PublicKernelCircuitPrivateInputs,
  PublicKernelCircuitPublicInputs,
  PublicKernelData,
  PublicKernelTailCircuitPrivateInputs,
  RECURSIVE_PROOF_LENGTH,
  ROLLUP_VK_TREE_HEIGHT,
  ReadRequest,
  RevertCode,
  RollupTypes,
  RootParityInput,
  RootParityInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  ScopedKeyValidationRequestAndGenerator,
  ScopedReadRequest,
  StateDiffHints,
  StateReference,
  TxContext,
  TxRequest,
  VK_TREE_HEIGHT,
  Vector,
  VerificationKey,
  VerificationKeyAsFields,
  VerificationKeyData,
  computeContractClassId,
  computePublicBytecodeCommitment,
  makeRecursiveProof,
  packBytecode,
} from '../index.js';
import { ContentCommitment, NUM_BYTES_PER_SHA256 } from '../structs/content_commitment.js';
import { Gas } from '../structs/gas.js';
import { GasFees } from '../structs/gas_fees.js';
import { GasSettings } from '../structs/gas_settings.js';
import { GlobalVariables } from '../structs/global_variables.js';
import { Header } from '../structs/header.js';
import { KernelCircuitPublicInputs } from '../structs/kernel/kernel_circuit_public_inputs.js';
import { KernelData } from '../structs/kernel/kernel_data.js';
import { RollupValidationRequests } from '../structs/rollup_validation_requests.js';
import { ValidationRequests } from '../structs/validation_requests.js';

/**
 * Creates an arbitrary side effect object with the given seed.
 * @param seed - The seed to use for generating the object.
 * @returns A side effect object.
 */
function makeLogHash(seed: number) {
  return new LogHash(fr(seed), seed + 1, fr(seed + 2));
}

function makeEncryptedLogHash(seed: number) {
  return new EncryptedLogHash(fr(seed), seed + 1, fr(seed + 2), fr(seed + 3));
}

function makeNoteLogHash(seed: number) {
  return new NoteLogHash(fr(seed + 3), seed + 1, fr(seed + 2), seed);
}

function makeNoteHash(seed: number) {
  return new NoteHash(fr(seed), seed + 1);
}

function makeNullifier(seed: number) {
  return new Nullifier(fr(seed), seed + 1, fr(seed + 2));
}

/**
 * Creates an arbitrary tx context with the given seed.
 * @param seed - The seed to use for generating the tx context.
 * @returns A tx context.
 */
export function makeTxContext(seed: number = 1): TxContext {
  // @todo @LHerskind should probably take value for chainId as it will be verified later.
  return new TxContext(new Fr(seed), Fr.ZERO, makeGasSettings());
}

/**
 * Creates arbitrary constant data with the given seed.
 * @param seed - The seed to use for generating the constant data.
 * @returns A constant data object.
 */
export function makeConstantData(seed = 1): CombinedConstantData {
  return new CombinedConstantData(makeHeader(seed, undefined), makeTxContext(seed + 4), makeGlobalVariables(seed + 5));
}

/**
 * Creates a default instance of gas settings. No seed value is used to ensure we allocate a sensible amount of gas for testing.
 */
export function makeGasSettings() {
  return GasSettings.default();
}

/**
 * Creates arbitrary selector from the given seed.
 * @param seed - The seed to use for generating the selector.
 * @returns A selector.
 */
export function makeSelector(seed: number): FunctionSelector {
  return new FunctionSelector(seed);
}

function makeReadRequest(n: number): ReadRequest {
  return new ReadRequest(new Fr(BigInt(n)), n + 1);
}

function makeScopedReadRequest(n: number): ScopedReadRequest {
  return new ScopedReadRequest(makeReadRequest(n), AztecAddress.fromBigInt(BigInt(n + 2)));
}

/**
 * Creates arbitrary KeyValidationRequest from the given seed.
 * @param seed - The seed to use for generating the KeyValidationRequest.
 * @returns A KeyValidationRequest.
 */
function makeKeyValidationRequests(seed: number): KeyValidationRequest {
  return new KeyValidationRequest(makePoint(seed), fr(seed + 2));
}

/**
 * Creates arbitrary KeyValidationRequestAndGenerator from the given seed.
 * @param seed - The seed to use for generating the KeyValidationRequestAndGenerator.
 * @returns A KeyValidationRequestAndGenerator.
 */
function makeKeyValidationRequestAndGenerators(seed: number): KeyValidationRequestAndGenerator {
  return new KeyValidationRequestAndGenerator(makeKeyValidationRequests(seed), fr(seed + 4));
}

/**
 * Creates arbitrary scoped ScopedKeyValidationRequestAndGenerator from the given seed.
 * @param seed - The seed to use for generating the ScopedKeyValidationRequestAndGenerator.
 * @returns A ScopedKeyValidationRequestAndGenerator.
 */
function makeScopedKeyValidationRequestAndGenerators(seed: number): ScopedKeyValidationRequestAndGenerator {
  return new ScopedKeyValidationRequestAndGenerator(
    makeKeyValidationRequestAndGenerators(seed),
    makeAztecAddress(seed + 4),
  );
}

/**
 * Creates arbitrary public data update request.
 * @param seed - The seed to use for generating the public data update request.
 * @returns A public data update request.
 */
export function makePublicDataUpdateRequest(seed = 1): PublicDataUpdateRequest {
  return new PublicDataUpdateRequest(fr(seed), fr(seed + 1), seed + 2);
}

/**
 * Creates empty public data update request.
 * @returns An empty public data update request.
 */
export function makeEmptyPublicDataUpdateRequest(): PublicDataUpdateRequest {
  return new PublicDataUpdateRequest(fr(0), fr(0), 0);
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
  return new ContractStorageUpdateRequest(fr(seed), fr(seed + 1), seed + 2);
}

/**
 * Creates arbitrary contract storage read.
 * @param seed - The seed to use for generating the contract storage read.
 * @returns A contract storage read.
 */
export function makeContractStorageRead(seed = 1): ContractStorageRead {
  return new ContractStorageRead(fr(seed), fr(seed + 1));
}

export function makeValidationRequests(seed = 1) {
  return new ValidationRequests(
    makeRollupValidationRequests(seed),
    makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, makeScopedReadRequest, seed + 0x80),
    makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, makeScopedReadRequest, seed + 0x90),
    makeTuple(MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, makeScopedReadRequest, seed + 0x95),
    makeTuple(MAX_KEY_VALIDATION_REQUESTS_PER_TX, makeScopedKeyValidationRequestAndGenerators, seed + 0x100),
    makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, makePublicDataRead, seed + 0xe00),
  );
}

export function makeRollupValidationRequests(seed = 1) {
  return new RollupValidationRequests(new MaxBlockNumber(true, new Fr(seed + 0x31415)));
}

export function makeCombinedConstantData(seed = 1): CombinedConstantData {
  return new CombinedConstantData(makeHeader(seed), makeTxContext(seed + 0x100), makeGlobalVariables(seed + 0x200));
}

/**
 * Creates arbitrary accumulated data.
 * @param seed - The seed to use for generating the accumulated data.
 * @returns An accumulated data.
 */
export function makeCombinedAccumulatedData(seed = 1, full = false): CombinedAccumulatedData {
  const tupleGenerator = full ? makeTuple : makeHalfFullTuple;

  return new CombinedAccumulatedData(
    tupleGenerator(MAX_NEW_NOTE_HASHES_PER_TX, fr, seed + 0x120, Fr.zero),
    tupleGenerator(MAX_NEW_NULLIFIERS_PER_TX, fr, seed + 0x200, Fr.zero),
    tupleGenerator(MAX_NEW_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x600, Fr.zero),
    fr(seed + 0x700), // note encrypted logs hash
    fr(seed + 0x800), // encrypted logs hash
    fr(seed + 0x900), // unencrypted logs hash
    fr(seed + 0xa00), // note_encrypted_log_preimages_length
    fr(seed + 0xb00), // encrypted_log_preimages_length
    fr(seed + 0xc00), // unencrypted_log_preimages_length
    tupleGenerator(
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      makePublicDataUpdateRequest,
      seed + 0xd00,
      PublicDataUpdateRequest.empty,
    ),
    makeGas(seed + 0xe00),
  );
}

export function makeGas(seed = 1) {
  return new Gas(seed, seed + 1);
}

/**
 * Creates arbitrary accumulated data.
 * @param seed - The seed to use for generating the accumulated data.
 * @returns An accumulated data.
 */
export function makePublicAccumulatedData(seed = 1, full = false): PublicAccumulatedData {
  const tupleGenerator = full ? makeTuple : makeHalfFullTuple;

  return new PublicAccumulatedData(
    tupleGenerator(MAX_NEW_NOTE_HASHES_PER_TX, makeNoteHash, seed + 0x120, NoteHash.empty),
    tupleGenerator(MAX_NEW_NULLIFIERS_PER_TX, makeNullifier, seed + 0x200, Nullifier.empty),
    tupleGenerator(MAX_NEW_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x600, Fr.zero),
    tupleGenerator(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, makeLogHash, seed + 0x700, LogHash.empty), // note encrypted logs hashes
    tupleGenerator(MAX_ENCRYPTED_LOGS_PER_TX, makeLogHash, seed + 0x800, LogHash.empty), // encrypted logs hashes
    tupleGenerator(MAX_UNENCRYPTED_LOGS_PER_TX, makeLogHash, seed + 0x900, LogHash.empty), // unencrypted logs hashes
    tupleGenerator(
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      makePublicDataUpdateRequest,
      seed + 0xd00,
      PublicDataUpdateRequest.empty,
    ),
    tupleGenerator(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x500, CallRequest.empty),
    makeGas(seed + 0x600),
  );
}

/**
 * Creates arbitrary call context.
 * @param seed - The seed to use for generating the call context.
 * @param storageContractAddress - The storage contract address set on the call context.
 * @returns A call context.
 */
export function makeCallContext(seed = 0, overrides: Partial<FieldsOf<CallContext>> = {}): CallContext {
  return CallContext.from({
    msgSender: makeAztecAddress(seed),
    storageContractAddress: makeAztecAddress(seed + 1),
    functionSelector: makeSelector(seed + 3),
    isStaticCall: false,
    isDelegateCall: false,
    sideEffectCounter: 0,
    ...overrides,
  });
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
    makeCallContext(seed, { storageContractAddress: storageContractAddress ?? makeAztecAddress(seed) }),
    fr(seed + 0x100),
    fr(seed + 0x200),
    tupleGenerator(MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, makeReadRequest, seed + 0x300, ReadRequest.empty),
    tupleGenerator(MAX_NULLIFIER_READ_REQUESTS_PER_CALL, makeReadRequest, seed + 0x400, ReadRequest.empty),
    tupleGenerator(MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL, makeReadRequest, seed + 0x420, ReadRequest.empty),
    tupleGenerator(MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL, makeReadRequest, seed + 0x440, ReadRequest.empty),
    tupleGenerator(
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
      makeContractStorageUpdateRequest,
      seed + 0x400,
      ContractStorageUpdateRequest.empty,
    ),
    tupleGenerator(MAX_PUBLIC_DATA_READS_PER_CALL, makeContractStorageRead, seed + 0x500, ContractStorageRead.empty),
    tupleGenerator(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, fr, seed + 0x600, Fr.zero),
    tupleGenerator(MAX_NEW_NOTE_HASHES_PER_CALL, makeNoteHash, seed + 0x700, NoteHash.empty),
    tupleGenerator(MAX_NEW_NULLIFIERS_PER_CALL, makeNullifier, seed + 0x800, Nullifier.empty),
    tupleGenerator(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, makeL2ToL1Message, seed + 0x900, L2ToL1Message.empty),
    fr(seed + 0xa00),
    fr(seed + 0xa01),
    tupleGenerator(MAX_UNENCRYPTED_LOGS_PER_CALL, makeLogHash, seed + 0x901, LogHash.empty),
    makeHeader(seed + 0xa00, undefined),
    makeGlobalVariables(seed + 0xa01),
    makeAztecAddress(seed + 0xb01),
    RevertCode.OK,
    makeGas(seed + 0xc00),
    makeGas(seed + 0xc00),
    fr(0),
  );
}

/**
 * Creates arbitrary public kernel circuit public inputs.
 * @param seed - The seed to use for generating the kernel circuit public inputs.
 * @returns Public kernel circuit public inputs.
 */
export function makePublicKernelCircuitPublicInputs(
  seed = 1,
  fullAccumulatedData = true,
): PublicKernelCircuitPublicInputs {
  const tupleGenerator = fullAccumulatedData ? makeTuple : makeHalfFullTuple;
  return new PublicKernelCircuitPublicInputs(
    makeValidationRequests(seed),
    makePublicAccumulatedData(seed, fullAccumulatedData),
    makePublicAccumulatedData(seed, fullAccumulatedData),
    makeConstantData(seed + 0x100),
    RevertCode.OK,
    tupleGenerator(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x600, CallRequest.empty),
    makeAztecAddress(seed + 0x700),
  );
}

/**
 * Creates arbitrary private kernel tail circuit public inputs.
 * @param seed - The seed to use for generating the kernel circuit public inputs.
 * @returns Private kernel tail circuit public inputs.
 */
export function makePrivateKernelTailCircuitPublicInputs(
  seed = 1,
  isForPublic = true,
): PrivateKernelTailCircuitPublicInputs {
  const forPublic = isForPublic
    ? new PartialPrivateTailPublicInputsForPublic(
        ValidationRequests.empty(),
        makePublicAccumulatedData(seed + 0x100, false),
        makePublicAccumulatedData(seed + 0x200, false),
        makeHalfFullTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x400, CallRequest.empty),
      )
    : undefined;
  const forRollup = !isForPublic
    ? new PartialPrivateTailPublicInputsForRollup(
        makeRollupValidationRequests(seed),
        makeCombinedAccumulatedData(seed + 0x100),
      )
    : undefined;
  return new PrivateKernelTailCircuitPublicInputs(
    makeConstantData(seed + 0x300),
    RevertCode.OK,
    makeAztecAddress(seed + 0x700),
    forPublic,
    forRollup,
  );
}

/**
 * Creates arbitrary public kernel circuit public inputs.
 * @param seed - The seed to use for generating the kernel circuit public inputs.
 * @returns Public kernel circuit public inputs.
 */
export function makeKernelCircuitPublicInputs(seed = 1, fullAccumulatedData = true): KernelCircuitPublicInputs {
  return new KernelCircuitPublicInputs(
    makeRollupValidationRequests(seed),
    makeCombinedAccumulatedData(seed, fullAccumulatedData),
    makeConstantData(seed + 0x100),
    makePartialStateReference(seed + 0x200),
    RevertCode.OK,
    makeAztecAddress(seed + 0x700),
  );
}

/**
 * Creates a public call request for testing.
 * @param seed - The seed.
 * @returns Public call request.
 */
export function makePublicCallRequest(seed = 1): PublicCallRequest {
  const childCallContext = makeCallContext(seed + 0x2, { storageContractAddress: makeAztecAddress(seed) });
  const parentCallContext = makeCallContext(seed + 0x3, { storageContractAddress: childCallContext.msgSender });

  return new PublicCallRequest(
    makeAztecAddress(seed),
    makeSelector(seed + 0x1),
    childCallContext,
    parentCallContext,
    makeTuple(ARGS_LENGTH, fr, seed + 0x10),
  );
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
 * Creates arbitrary/mocked verification key in fields format.
 * @returns A verification key as fields object
 */
export function makeVerificationKeyAsFields(): VerificationKeyAsFields {
  return VerificationKeyAsFields.makeFake();
}

/**
 * Creates arbitrary/mocked verification key.
 * @returns A verification key object
 */
export function makeVerificationKey(): VerificationKey {
  return VerificationKey.makeFake();
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
 * Makes arbitrary public kernel data.
 * @param seed - The seed to use for generating the previous kernel data.
 * @param kernelPublicInputs - The public kernel public inputs to use for generating the public kernel data.
 * @returns A previous kernel data.
 */
export function makePublicKernelData(seed = 1, kernelPublicInputs?: PublicKernelCircuitPublicInputs): PublicKernelData {
  return new PublicKernelData(
    kernelPublicInputs ?? makePublicKernelCircuitPublicInputs(seed, true),
    makeRecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH, seed + 0x80),
    VerificationKeyData.makeFake(),
    0x42,
    makeTuple(VK_TREE_HEIGHT, fr, 0x1000),
  );
}

/**
 * Makes arbitrary public kernel data.
 * @param seed - The seed to use for generating the previous kernel data.
 * @param kernelPublicInputs - The public kernel public inputs to use for generating the public kernel data.
 * @returns A previous kernel data.
 */
export function makeRollupKernelData(seed = 1, kernelPublicInputs?: KernelCircuitPublicInputs): KernelData {
  return new KernelData(
    kernelPublicInputs ?? makeKernelCircuitPublicInputs(seed, true),
    makeRecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH, seed + 0x80),
    VerificationKeyData.makeFake(),
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
  return new Proof(Buffer.alloc(16, seed), 0);
}

/**
 * Makes arbitrary call stack item.
 * @param seed - The seed to use for generating the call stack item.
 * @returns A call stack item.
 */
export function makeCallerContext(seed = 1): CallerContext {
  return new CallerContext(makeAztecAddress(seed), makeAztecAddress(seed + 0x1), false);
}

/**
 * Makes arbitrary call stack item.
 * @param seed - The seed to use for generating the call stack item.
 * @returns A call stack item.
 */
export function makeCallRequest(seed = 1): CallRequest {
  return new CallRequest(fr(seed), makeAztecAddress(seed + 0x1), makeCallerContext(seed + 0x2), fr(0), fr(0));
}

function makePrivateCallRequest(seed = 1): PrivateCallRequest {
  return new PrivateCallRequest(fr(seed), makeCallerContext(seed + 0x2), seed + 0x10, seed + 0x11);
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
    new FunctionData(makeSelector(seed + 0x1), /*isPrivate=*/ false),
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
  );

  return publicCallData;
}

/**
 * Makes arbitrary public kernel inputs.
 * @param seed - The seed to use for generating the public kernel inputs.
 * @returns Public kernel inputs.
 */
export function makePublicKernelCircuitPrivateInputs(seed = 1): PublicKernelCircuitPrivateInputs {
  return new PublicKernelCircuitPrivateInputs(makePublicKernelData(seed), makePublicCallData(seed + 0x1000));
}

export function makeCombineHints(seed = 1): CombineHints {
  return CombineHints.from({
    sortedNoteHashes: makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, makeNoteHash, seed + 0x100),
    sortedNoteHashesIndexes: makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, i => i, seed + 0x200),
    sortedUnencryptedLogsHashes: makeTuple(MAX_UNENCRYPTED_LOGS_PER_TX, makeLogHash, seed + 0x300),
    sortedUnencryptedLogsHashesIndexes: makeTuple(MAX_UNENCRYPTED_LOGS_PER_TX, i => i, seed + 0x400),
    sortedPublicDataUpdateRequests: makeTuple(
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      makePublicDataUpdateRequest,
      seed + 0x300,
    ),
    sortedPublicDataUpdateRequestsIndexes: makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => i, seed + 0x400),
    dedupedPublicDataUpdateRequests: makeTuple(
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      makePublicDataUpdateRequest,
      seed + 0x500,
    ),
    dedupedPublicDataUpdateRequestsRuns: makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => i, seed + 0x600),
  });
}

/**
 * Makes arbitrary public kernel tail inputs.
 * @param seed - The seed to use for generating the public kernel inputs.
 * @returns Public kernel inputs.
 */
export function makePublicKernelTailCircuitPrivateInputs(seed = 1): PublicKernelTailCircuitPrivateInputs {
  return new PublicKernelTailCircuitPrivateInputs(
    makePublicKernelData(seed),
    NullifierReadRequestHintsBuilder.empty(MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX),
    NullifierNonExistentReadRequestHintsBuilder.empty(),
    makeTuple(MAX_PUBLIC_DATA_HINTS, PublicDataHint.empty, seed + 0x100),
    PublicDataReadRequestHintsBuilder.empty(),
    makePartialStateReference(seed + 0x200),
    makeCombineHints(seed + 0x300),
  );
}

/**
 * Makes arbitrary public kernel private inputs.
 * @param seed - The seed to use for generating the public kernel inputs.
 * @param tweak - An optional function to tweak the output before computing hashes.
 * @returns Public kernel inputs.
 */
export function makePublicKernelInputsWithTweak(
  seed = 1,
  tweak?: (publicKernelInputs: PublicKernelCircuitPrivateInputs) => void,
): PublicKernelCircuitPrivateInputs {
  const kernelCircuitPublicInputs = makePublicKernelCircuitPublicInputs(seed, false);
  const previousKernel = makePublicKernelData(seed, kernelCircuitPublicInputs);
  const publicCall = makePublicCallData(seed + 0x1000);
  const publicKernelInputs = new PublicKernelCircuitPrivateInputs(previousKernel, publicCall);
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
    functionData: new FunctionData(makeSelector(seed + 0x100), /*isPrivate=*/ true),
    argsHash: fr(seed + 0x200),
    txContext: makeTxContext(seed + 0x400),
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
    new FunctionData(makeSelector(seed + 0x1), /*isPrivate=*/ true),
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
    maxBlockNumber: new MaxBlockNumber(true, new Fr(seed + 0x31415)),
    callContext: makeCallContext(seed, { isDelegateCall: true, isStaticCall: true }),
    argsHash: fr(seed + 0x100),
    returnsHash: fr(seed + 0x200),
    minRevertibleSideEffectCounter: fr(0),
    noteHashReadRequests: makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, makeReadRequest, seed + 0x300),
    nullifierReadRequests: makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_CALL, makeReadRequest, seed + 0x310),
    keyValidationRequestsAndGenerators: makeTuple(
      MAX_KEY_VALIDATION_REQUESTS_PER_CALL,
      makeKeyValidationRequestAndGenerators,
      seed + 0x320,
    ),
    newNoteHashes: makeTuple(MAX_NEW_NOTE_HASHES_PER_CALL, makeNoteHash, seed + 0x400),
    newNullifiers: makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, makeNullifier, seed + 0x500),
    privateCallRequests: makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, makePrivateCallRequest, seed + 0x600),
    publicCallStackHashes: makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, fr, seed + 0x700),
    publicTeardownFunctionHash: fr(seed + 0x800),
    newL2ToL1Msgs: makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, makeL2ToL1Message, seed + 0x800),
    startSideEffectCounter: fr(seed + 0x849),
    endSideEffectCounter: fr(seed + 0x850),
    noteEncryptedLogsHashes: makeTuple(MAX_NOTE_ENCRYPTED_LOGS_PER_CALL, makeNoteLogHash, seed + 0x875),
    encryptedLogsHashes: makeTuple(MAX_ENCRYPTED_LOGS_PER_CALL, makeEncryptedLogHash, seed + 0x900),
    unencryptedLogsHashes: makeTuple(MAX_UNENCRYPTED_LOGS_PER_CALL, makeLogHash, seed + 0xa00),
    historicalHeader: makeHeader(seed + 0xd00, undefined),
    txContext: makeTxContext(seed + 0x1400),
    isFeePayer: false,
  });
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
    return new GlobalVariables(
      fr(seed),
      fr(seed + 1),
      fr(blockNumber),
      fr(seed + 3),
      EthAddress.fromField(fr(seed + 4)),
      AztecAddress.fromField(fr(seed + 5)),
      makeGasFees(seed + 6),
    );
  }
  return new GlobalVariables(
    fr(seed),
    fr(seed + 1),
    fr(seed + 2),
    fr(seed + 3),
    EthAddress.fromField(fr(seed + 4)),
    AztecAddress.fromField(fr(seed + 5)),
    makeGasFees(seed + 6),
  );
}

export function makeGasFees(seed = 1) {
  return new GasFees(fr(seed), fr(seed + 1));
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
  return EthAddress.fromField(fr(seed));
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
  return AztecAddress.fromField(fr(seed));
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
    makeConstantBaseRollupData(seed + 0x200, globalVariables),
    makePartialStateReference(seed + 0x300),
    makePartialStateReference(seed + 0x400),
    fr(seed + 0x901),
    fr(seed + 0x902),
    fr(seed + 0x903),
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
    makeRecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH, seed + 0x50),
    VerificationKeyAsFields.makeFake(),
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
    makeRootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH, seed + 0x2000),
    makeTuple(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, fr, 0x2100),
    makeTuple(L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH, fr, 0x2100),
    makeAppendOnlyTreeSnapshot(seed + 0x2200),
    makeAppendOnlyTreeSnapshot(seed + 0x2200),
    makeTuple(ARCHIVE_HEIGHT, fr, 0x2400),
  );
}

export function makeRootParityInput<PROOF_LENGTH extends number>(
  proofSize: PROOF_LENGTH,
  seed = 0,
): RootParityInput<PROOF_LENGTH> {
  return new RootParityInput<PROOF_LENGTH>(
    makeRecursiveProof<PROOF_LENGTH>(proofSize, seed),
    VerificationKeyAsFields.makeFake(seed + 0x100),
    makeParityPublicInputs(seed + 0x200),
  );
}

export function makeParityPublicInputs(seed = 0): ParityPublicInputs {
  return new ParityPublicInputs(new Fr(BigInt(seed + 0x200)), new Fr(BigInt(seed + 0x300)));
}

export function makeBaseParityInputs(seed = 0): BaseParityInputs {
  return new BaseParityInputs(makeTuple(NUM_MSGS_PER_BASE_PARITY, fr, seed + 0x3000));
}

export function makeRootParityInputs(seed = 0): RootParityInputs {
  return new RootParityInputs(
    makeTuple(
      NUM_BASE_PARITY_PER_ROOT_PARITY,
      () => makeRootParityInput<typeof RECURSIVE_PROOF_LENGTH>(RECURSIVE_PROOF_LENGTH),
      seed + 0x4100,
    ),
  );
}

/**
 * Makes root rollup public inputs.
 * @param seed - The seed to use for generating the root rollup public inputs.
 * @param blockNumber - The block number to use in the global variables of a header.
 * @returns A root rollup public inputs.
 */
export function makeRootRollupPublicInputs(
  seed = 0,
  blockNumber: number | undefined = undefined,
): RootRollupPublicInputs {
  return RootRollupPublicInputs.from({
    archive: makeAppendOnlyTreeSnapshot(seed + 0x100),
    header: makeHeader(seed + 0x200, blockNumber),
  });
}

/**
 * Makes content commitment
 */
export function makeContentCommitment(seed = 0, txsEffectsHash: Buffer | undefined = undefined): ContentCommitment {
  return new ContentCommitment(
    new Fr(seed),
    txsEffectsHash ?? toBufferBE(BigInt(seed + 0x100), NUM_BYTES_PER_SHA256),
    toBufferBE(BigInt(seed + 0x200), NUM_BYTES_PER_SHA256),
    toBufferBE(BigInt(seed + 0x300), NUM_BYTES_PER_SHA256),
  );
}

/**
 * Makes header.
 */
export function makeHeader(
  seed = 0,
  blockNumber: number | undefined = undefined,
  txsEffectsHash: Buffer | undefined = undefined,
): Header {
  return new Header(
    makeAppendOnlyTreeSnapshot(seed + 0x100),
    makeContentCommitment(seed + 0x200, txsEffectsHash),
    makeStateReference(seed + 0x600),
    makeGlobalVariables((seed += 0x700), blockNumber),
    fr(seed + 0x800),
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
 * Makes arbitrary L2 to L1 message.
 * @param seed - The seed to use for generating the state reference.
 * @returns L2 to L1 message.
 */
export function makeL2ToL1Message(seed = 0): L2ToL1Message {
  const recipient = EthAddress.fromField(new Fr(seed));
  const content = new Fr(seed + 1);

  return new L2ToL1Message(recipient, content, seed + 2);
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

  const publicDataSiblingPath = makeTuple(PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, fr, 0x8000);

  return new StateDiffHints(
    nullifierPredecessorPreimages,
    nullifierPredecessorMembershipWitnesses,
    sortedNullifiers,
    sortedNullifierIndexes,
    noteHashSubtreeSiblingPath,
    nullifierSubtreeSiblingPath,
    publicDataSiblingPath,
  );
}

/**
 * Makes arbitrary base rollup inputs.
 * @param seed - The seed to use for generating the base rollup inputs.
 * @returns A base rollup inputs.
 */
export function makeBaseRollupInputs(seed = 0): BaseRollupInputs {
  const kernelData = makeRollupKernelData(seed);

  const start = makePartialStateReference(seed + 0x100);

  const stateDiffHints = makeStateDiffHints(seed + 0x600);

  const sortedPublicDataWrites = makeTuple(
    MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    makePublicDataTreeLeaf,
    seed + 0x8000,
  );

  const sortedPublicDataWritesIndexes = makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, i => i, 0);

  const lowPublicDataWritesPreimages = makeTuple(
    MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    makePublicDataTreeLeafPreimage,
    seed + 0x8200,
  );

  const lowPublicDataWritesMembershipWitnesses = makeTuple(
    MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    i => makeMembershipWitness(PUBLIC_DATA_TREE_HEIGHT, i),
    seed + 0x8400,
  );

  const archiveRootMembershipWitness = makeMembershipWitness(ARCHIVE_HEIGHT, seed + 0x9000);

  const constants = makeConstantBaseRollupData(0x100);

  const feePayerGasTokenBalanceReadHint = PublicDataHint.empty();

  return BaseRollupInputs.from({
    kernelData,
    start,
    stateDiffHints,
    sortedPublicDataWrites,
    sortedPublicDataWritesIndexes,
    lowPublicDataWritesPreimages,
    lowPublicDataWritesMembershipWitnesses,
    archiveRootMembershipWitness,
    constants,
    feePayerGasTokenBalanceReadHint,
  });
}

export function makeExecutablePrivateFunctionWithMembershipProof(
  seed = 0,
): ExecutablePrivateFunctionWithMembershipProof {
  return {
    selector: makeSelector(seed),
    bytecode: makeBytes(100, seed + 1),
    artifactTreeSiblingPath: makeTuple(3, fr, seed + 2),
    artifactTreeLeafIndex: seed + 2,
    privateFunctionTreeSiblingPath: makeTuple(3, fr, seed + 3),
    privateFunctionTreeLeafIndex: seed + 3,
    artifactMetadataHash: fr(seed + 4),
    functionMetadataHash: fr(seed + 5),
    unconstrainedFunctionsArtifactTreeRoot: fr(seed + 6),
    vkHash: fr(seed + 7),
  };
}

export function makeUnconstrainedFunctionWithMembershipProof(seed = 0): UnconstrainedFunctionWithMembershipProof {
  return {
    selector: makeSelector(seed),
    bytecode: makeBytes(100, seed + 1),
    artifactTreeSiblingPath: makeTuple(3, fr, seed + 2),
    artifactTreeLeafIndex: seed + 2,
    artifactMetadataHash: fr(seed + 4),
    functionMetadataHash: fr(seed + 5),
    privateFunctionsArtifactTreeRoot: fr(seed + 6),
  };
}

export function makeContractClassPublic(seed = 0): ContractClassPublic {
  const artifactHash = fr(seed + 1);
  const publicFunctions = makeTuple(3, makeContractClassPublicFunction, seed + 2);
  const privateFunctionsRoot = fr(seed + 3);
  const packedBytecode = packBytecode(publicFunctions);
  const publicBytecodeCommitment = computePublicBytecodeCommitment(packedBytecode);
  const id = computeContractClassId({ artifactHash, privateFunctionsRoot, publicBytecodeCommitment });
  return {
    id,
    artifactHash,
    packedBytecode,
    privateFunctionsRoot,
    publicFunctions,
    privateFunctions: [],
    unconstrainedFunctions: [],
    version: 1,
  };
}

function makeContractClassPublicFunction(seed = 0): PublicFunction {
  return {
    selector: FunctionSelector.fromField(fr(seed + 1)),
    bytecode: makeBytes(100, seed + 2),
  };
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function makeContractClassPrivateFunction(seed = 0): PrivateFunction {
  return {
    selector: FunctionSelector.fromField(fr(seed + 1)),
    vkHash: fr(seed + 2),
  };
}

export function makeArray<T extends Bufferable>(length: number, fn: (i: number) => T, offset = 0) {
  return Array.from({ length }, (_: any, i: number) => fn(i + offset));
}

export function makeVector<T extends Bufferable>(length: number, fn: (i: number) => T, offset = 0) {
  return new Vector(makeArray(length, fn, offset));
}

/**
 * Makes arbitrary AvmKeyValueHint.
 * @param seed - The seed to use for generating the state reference.
 * @returns AvmKeyValueHint.
 */
export function makeAvmKeyValueHint(seed = 0): AvmKeyValueHint {
  return new AvmKeyValueHint(new Fr(seed), new Fr(seed + 1));
}

/**
 * Makes arbitrary AvmExternalCallHint.
 * @param seed - The seed to use for generating the state reference.
 * @returns AvmKeyValueHint.
 */
export function makeAvmExternalCallHint(seed = 0): AvmExternalCallHint {
  return new AvmExternalCallHint(
    new Fr(seed % 2),
    makeArray(seed + 10, i => new Fr(i), seed + 0x1000),
    new Gas(seed + 0x200, seed),
  );
}

/**
 * Creates arbitrary AvmExecutionHints.
 * @param seed - The seed to use for generating the hints.
 * @returns the execution hints.
 */
export function makeAvmExecutionHints(
  seed = 0,
  overrides: Partial<FieldsOf<AvmExecutionHints>> = {},
): AvmExecutionHints {
  const lengthOffset = 10;
  const lengthSeedMod = 10;
  const baseLength = lengthOffset + (seed % lengthSeedMod);

  return AvmExecutionHints.from({
    storageValues: makeVector(baseLength, makeAvmKeyValueHint, seed + 0x4200),
    noteHashExists: makeVector(baseLength + 1, makeAvmKeyValueHint, seed + 0x4300),
    nullifierExists: makeVector(baseLength + 2, makeAvmKeyValueHint, seed + 0x4400),
    l1ToL2MessageExists: makeVector(baseLength + 3, makeAvmKeyValueHint, seed + 0x4500),
    externalCalls: makeVector(baseLength + 4, makeAvmExternalCallHint, seed + 0x4600),
    ...overrides,
  });
}

/**
 * Creates arbitrary AvmCircuitInputs.
 * @param seed - The seed to use for generating the hints.
 * @returns the execution hints.
 */
export function makeAvmCircuitInputs(seed = 0, overrides: Partial<FieldsOf<AvmCircuitInputs>> = {}): AvmCircuitInputs {
  return AvmCircuitInputs.from({
    bytecode: makeBytes(seed + 100, seed),
    calldata: makeArray(seed + 10, i => new Fr(i), seed + 0x1000),
    publicInputs: makePublicCircuitPublicInputs(seed + 0x2000),
    avmHints: makeAvmExecutionHints(seed + 0x3000),
    ...overrides,
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

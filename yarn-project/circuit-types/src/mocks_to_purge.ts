import {
  ARGS_LENGTH,
  AggregationObject,
  AztecAddress,
  CallContext,
  CallRequest,
  CallerContext,
  CombinedConstantData,
  EthAddress,
  Fq,
  Fr,
  FunctionData,
  FunctionSelector,
  G1AffineElement,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX,
  MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
  MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_REVERTIBLE_NOTE_HASHES_PER_TX,
  MAX_REVERTIBLE_NULLIFIERS_PER_TX,
  MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  NUM_FIELDS_PER_SHA256,
  Point,
  PrivateAccumulatedNonRevertibleData,
  PrivateAccumulatedRevertibleData,
  PrivateKernelTailCircuitPublicInputs,
  PublicCallRequest,
  RevertCode,
  SideEffect,
  SideEffectLinkedToNoteHash,
  TxContext,
} from '@aztec/circuits.js';
import { makeRollupValidationRequests } from '@aztec/circuits.js/testing';
import { makeHalfFullTuple, makeTuple, range } from '@aztec/foundation/array';

import { makeHeader } from './l2_block_code_to_purge.js';

/**
 * Creates arbitrary private kernel tail circuit public inputs.
 * @param seed - The seed to use for generating the kernel circuit public inputs.
 * @returns Private kernel tail circuit public inputs.
 */
export function makePrivateKernelTailCircuitPublicInputs(seed = 1, full = true): PrivateKernelTailCircuitPublicInputs {
  return new PrivateKernelTailCircuitPublicInputs(
    makeAggregationObject(seed),
    makeRollupValidationRequests(seed),
    makeAccumulatedNonRevertibleData(seed + 0x100, full),
    makeFinalAccumulatedData(seed + 0x200, full),
    makeConstantData(seed + 0x300),
    true,
    true,
    true,
  );
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
 * Creates arbitrary accumulated data for a Tx's non-revertible side effects.
 * @param seed - The seed to use for generating the data.
 * @returns An instance of AccumulatedNonRevertibleData.
 */
export function makeAccumulatedNonRevertibleData(seed = 1, full = false): PrivateAccumulatedNonRevertibleData {
  const tupleGenerator = full ? makeTuple : makeHalfFullTuple;

  return new PrivateAccumulatedNonRevertibleData(
    RevertCode.OK,
    tupleGenerator(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, sideEffectFromNumber, seed + 0x101),
    tupleGenerator(MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, sideEffectLinkedFromNumber, seed + 0x201),
    tupleGenerator(MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x501),
  );
}

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

/**
 * Makes arbitrary call stack item.
 * @param seed - The seed to use for generating the call stack item.
 * @returns A call stack item.
 */
export function makeCallRequest(seed = 1): CallRequest {
  return new CallRequest(fr(seed), makeAztecAddress(seed + 0x1), makeCallerContext(seed + 0x2), fr(0), fr(0));
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
 * Makes arbitrary call stack item.
 * @param seed - The seed to use for generating the call stack item.
 * @returns A call stack item.
 */
export function makeCallerContext(seed = 1): CallerContext {
  return new CallerContext(makeAztecAddress(seed), makeAztecAddress(seed + 0x1));
}

/**
 * Creates arbitrary final accumulated data.
 * @param seed - The seed to use for generating the final accumulated data.
 * @returns A final accumulated data.
 */
export function makeFinalAccumulatedData(seed = 1, full = false): PrivateAccumulatedRevertibleData {
  const tupleGenerator = full ? makeTuple : makeHalfFullTuple;

  return new PrivateAccumulatedRevertibleData(
    tupleGenerator(MAX_REVERTIBLE_NOTE_HASHES_PER_TX, sideEffectFromNumber, seed + 0x100),
    tupleGenerator(MAX_REVERTIBLE_NULLIFIERS_PER_TX, sideEffectLinkedFromNumber, seed + 0x200),
    tupleGenerator(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x400),
    tupleGenerator(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, makeCallRequest, seed + 0x500),
    tupleGenerator(MAX_NEW_L2_TO_L1_MSGS_PER_TX, fr, seed + 0x600),
    tupleGenerator(NUM_FIELDS_PER_SHA256, fr, seed + 0x700), // encrypted logs hash
    tupleGenerator(NUM_FIELDS_PER_SHA256, fr, seed + 0x800), // unencrypted logs hash
    fr(seed + 0x900), // encrypted_log_preimages_length
    fr(seed + 0xa00), // unencrypted_log_preimages_length
  );
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
 * Creates arbitrary constant data with the given seed.
 * @param seed - The seed to use for generating the constant data.
 * @returns A constant data object.
 */
export function makeConstantData(seed = 1): CombinedConstantData {
  return new CombinedConstantData(makeHeader(seed, undefined), makeTxContext(seed + 4));
}

/**
 * Creates an arbitrary tx context with the given seed.
 * @param seed - The seed to use for generating the tx context.
 * @returns A tx context.
 */
export function makeTxContext(_seed: number): TxContext {
  // @todo @LHerskind should probably take value for chainId as it will be verified later.
  return new TxContext(false, false, Fr.ZERO, Fr.ZERO);
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
 * Creates a public call request for testing.
 * @param seed - The seed.
 * @returns Public call request.
 */
export function makePublicCallRequest(seed = 1): PublicCallRequest {
  const childCallContext = makeCallContext(seed + 0x2, makeAztecAddress(seed));
  const parentCallContext = CallContext.from({
    msgSender: makeAztecAddress(seed + 0x3),
    storageContractAddress: childCallContext.msgSender,
    portalContractAddress: makeEthAddress(seed + 2),
    functionSelector: makeSelector(seed + 3),
    isStaticCall: false,
    isDelegateCall: false,
    sideEffectCounter: 0,
  });
  return new PublicCallRequest(
    makeAztecAddress(seed),
    new FunctionData(makeSelector(seed + 0x1), false),
    childCallContext,
    parentCallContext,
    makeTuple(ARGS_LENGTH, fr, seed + 0x10),
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
    0,
  );
}

/**
 * Creates arbitrary selector from the given seed.
 * @param seed - The seed to use for generating the selector.
 * @returns A selector.
 */
export function makeSelector(seed: number): FunctionSelector {
  return new FunctionSelector(seed);
}

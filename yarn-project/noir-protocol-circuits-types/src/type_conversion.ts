import {
  AppendOnlyTreeSnapshot,
  AztecAddress,
  BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  CallContext,
  CallRequest,
  CallerContext,
  type CombineHints,
  CombinedAccumulatedData,
  CombinedConstantData,
  ConstantRollupData,
  ContentCommitment,
  type ContractStorageRead,
  type ContractStorageUpdateRequest,
  type EmptyNestedData,
  EncryptedLogHash,
  EthAddress,
  Fr,
  FunctionData,
  FunctionSelector,
  Gas,
  GasFees,
  GasSettings,
  GlobalVariables,
  GrumpkinScalar,
  Header,
  KernelCircuitPublicInputs,
  type KernelData,
  type KeyValidationHint,
  KeyValidationRequest,
  KeyValidationRequestAndGenerator,
  L2ToL1Message,
  type LeafDataReadHint,
  LogHash,
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_L2_TO_L1_MSGS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
  MaxBlockNumber,
  type MembershipWitness,
  type MergeRollupInputs,
  type NESTED_RECURSIVE_PROOF_LENGTH,
  type NOTE_HASH_TREE_HEIGHT,
  type NULLIFIER_TREE_HEIGHT,
  NUM_BYTES_PER_SHA256,
  type NonMembershipHint,
  NoteHash,
  type NoteHashReadRequestHints,
  NoteLogHash,
  Nullifier,
  type NullifierLeafPreimage,
  type NullifierNonExistentReadRequestHints,
  type NullifierReadRequestHints,
  type PUBLIC_DATA_TREE_HEIGHT,
  ParityPublicInputs,
  PartialPrivateTailPublicInputsForPublic,
  PartialPrivateTailPublicInputsForRollup,
  PartialStateReference,
  type PendingReadHint,
  Point,
  type PreviousRollupData,
  PrivateAccumulatedData,
  type PrivateCallData,
  PrivateCallRequest,
  type PrivateCallStackItem,
  type PrivateCircuitPublicInputs,
  PrivateKernelCircuitPublicInputs,
  type PrivateKernelData,
  type PrivateKernelEmptyInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelResetCircuitPrivateInputs,
  type PrivateKernelResetHints,
  type PrivateKernelTailCircuitPrivateInputs,
  PrivateKernelTailCircuitPublicInputs,
  PublicAccumulatedData,
  type PublicCallData,
  type PublicCallStackItem,
  type PublicCircuitPublicInputs,
  type PublicDataHint,
  PublicDataRead,
  type PublicDataReadRequestHints,
  type PublicDataTreeLeaf,
  type PublicDataTreeLeafPreimage,
  PublicDataUpdateRequest,
  type PublicKernelCircuitPrivateInputs,
  PublicKernelCircuitPublicInputs,
  type PublicKernelData,
  type PublicKernelTailCircuitPrivateInputs,
  type RECURSIVE_PROOF_LENGTH,
  ReadRequest,
  type ReadRequestStatus,
  type RecursiveProof,
  RevertCode,
  RollupValidationRequests,
  type RootParityInput,
  type RootParityInputs,
  type RootRollupInputs,
  RootRollupPublicInputs,
  ScopedEncryptedLogHash,
  ScopedKeyValidationRequestAndGenerator,
  ScopedL2ToL1Message,
  ScopedLogHash,
  ScopedNoteHash,
  ScopedNullifier,
  ScopedPrivateCallRequest,
  ScopedReadRequest,
  type SettledReadHint,
  type StateDiffHints,
  StateReference,
  TxContext,
  type TxRequest,
  ValidationRequests,
  type VerificationKeyAsFields,
} from '@aztec/circuits.js';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { type Tuple, mapTuple, toTruncField } from '@aztec/foundation/serialize';

import type {
  AppendOnlyTreeSnapshot as AppendOnlyTreeSnapshotNoir,
  BaseOrMergeRollupPublicInputs as BaseOrMergeRollupPublicInputsNoir,
  BaseParityInputs as BaseParityInputsNoir,
  BaseRollupInputs as BaseRollupInputsNoir,
  CallContext as CallContextNoir,
  CallRequest as CallRequestNoir,
  CallerContext as CallerContextNoir,
  CombineHints as CombineHintsNoir,
  CombinedAccumulatedData as CombinedAccumulatedDataNoir,
  CombinedConstantData as CombinedConstantDataNoir,
  ConstantRollupData as ConstantRollupDataNoir,
  ContentCommitment as ContentCommitmentNoir,
  EmptyNestedCircuitPublicInputs as EmptyNestedDataNoir,
  EncryptedLogHash as EncryptedLogHashNoir,
  Field,
  FixedLengthArray,
  FunctionData as FunctionDataNoir,
  FunctionSelector as FunctionSelectorNoir,
  GasFees as GasFeesNoir,
  Gas as GasNoir,
  GasSettings as GasSettingsNoir,
  GlobalVariables as GlobalVariablesNoir,
  EmbeddedCurveScalar as GrumpkinScalarNoir,
  Header as HeaderNoir,
  KernelCircuitPublicInputs as KernelCircuitPublicInputsNoir,
  KernelData as KernelDataNoir,
  KeyValidationHint as KeyValidationHintNoir,
  KeyValidationRequestAndGenerator as KeyValidationRequestAndGeneratorNoir,
  KeyValidationRequest as KeyValidationRequestsNoir,
  L2ToL1Message as L2ToL1MessageNoir,
  LeafDataReadHint as LeafDataReadHintNoir,
  LogHash as LogHashNoir,
  MaxBlockNumber as MaxBlockNumberNoir,
  MembershipWitness as MembershipWitnessNoir,
  MergeRollupInputs as MergeRollupInputsNoir,
  AztecAddress as NoirAztecAddress,
  EthAddress as NoirEthAddress,
  Field as NoirField,
  EmbeddedCurvePoint as NoirPoint,
  NoteHashLeafPreimage as NoteHashLeafPreimageNoir,
  NoteHash as NoteHashNoir,
  NoteHashReadRequestHints as NoteHashReadRequestHintsNoir,
  NoteHashSettledReadHint as NoteHashSettledReadHintNoir,
  NoteLogHash as NoteLogHashNoir,
  NullifierLeafPreimage as NullifierLeafPreimageNoir,
  Nullifier as NullifierNoir,
  NullifierNonExistentReadRequestHints as NullifierNonExistentReadRequestHintsNoir,
  NullifierNonMembershipHint as NullifierNonMembershipHintNoir,
  NullifierReadRequestHints as NullifierReadRequestHintsNoir,
  NullifierSettledReadHint as NullifierSettledReadHintNoir,
  ParityPublicInputs as ParityPublicInputsNoir,
  RootParityInput as ParityRootParityInputNoir,
  PartialStateReference as PartialStateReferenceNoir,
  PendingReadHint as PendingReadHintNoir,
  PreviousRollupData as PreviousRollupDataNoir,
  PrivateAccumulatedData as PrivateAccumulatedDataNoir,
  PrivateCallData as PrivateCallDataNoir,
  PrivateCallRequest as PrivateCallRequestNoir,
  PrivateCallStackItem as PrivateCallStackItemNoir,
  PrivateCircuitPublicInputs as PrivateCircuitPublicInputsNoir,
  PrivateKernelCircuitPublicInputs as PrivateKernelCircuitPublicInputsNoir,
  PrivateKernelData as PrivateKernelDataNoir,
  PrivateKernelEmptyPrivateInputs as PrivateKernelEmptyPrivateInputsNoir,
  PrivateKernelInitCircuitPrivateInputs as PrivateKernelInitCircuitPrivateInputsNoir,
  PrivateKernelInnerCircuitPrivateInputs as PrivateKernelInnerCircuitPrivateInputsNoir,
  PrivateKernelResetCircuitPrivateInputs as PrivateKernelResetCircuitPrivateInputsNoir,
  PrivateKernelResetHints as PrivateKernelResetHintsNoir,
  PrivateKernelTailCircuitPrivateInputs as PrivateKernelTailCircuitPrivateInputsNoir,
  PrivateKernelTailToPublicCircuitPrivateInputs as PrivateKernelTailToPublicCircuitPrivateInputsNoir,
  PublicAccumulatedData as PublicAccumulatedDataNoir,
  PublicCallData as PublicCallDataNoir,
  PublicCallStackItem as PublicCallStackItemNoir,
  PublicCircuitPublicInputs as PublicCircuitPublicInputsNoir,
  PublicDataHint as PublicDataHintNoir,
  PublicDataRead as PublicDataReadNoir,
  PublicDataReadRequestHints as PublicDataReadRequestHintsNoir,
  PublicDataTreeLeaf as PublicDataTreeLeafNoir,
  PublicDataTreeLeafPreimage as PublicDataTreeLeafPreimageNoir,
  PublicDataUpdateRequest as PublicDataUpdateRequestNoir,
  PublicKernelCircuitPublicInputs as PublicKernelCircuitPublicInputsNoir,
  PublicKernelData as PublicKernelDataNoir,
  PublicKernelSetupCircuitPrivateInputs as PublicKernelSetupCircuitPrivateInputsNoir,
  PublicKernelTailCircuitPrivateInputs as PublicKernelTailCircuitPrivateInputsNoir,
  ReadRequest as ReadRequestNoir,
  ReadRequestStatus as ReadRequestStatusNoir,
  RollupValidationRequests as RollupValidationRequestsNoir,
  RootParityInputs as RootParityInputsNoir,
  RootRollupInputs as RootRollupInputsNoir,
  RootRollupParityInput as RootRollupParityInputNoir,
  RootRollupPublicInputs as RootRollupPublicInputsNoir,
  ScopedEncryptedLogHash as ScopedEncryptedLogHashNoir,
  ScopedKeyValidationRequestAndGenerator as ScopedKeyValidationRequestAndGeneratorNoir,
  ScopedL2ToL1Message as ScopedL2ToL1MessageNoir,
  ScopedLogHash as ScopedLogHashNoir,
  ScopedNoteHash as ScopedNoteHashNoir,
  ScopedNullifier as ScopedNullifierNoir,
  ScopedPrivateCallRequest as ScopedPrivateCallRequestNoir,
  ScopedReadRequest as ScopedReadRequestNoir,
  StateDiffHints as StateDiffHintsNoir,
  StateReference as StateReferenceNoir,
  StorageRead as StorageReadNoir,
  StorageUpdateRequest as StorageUpdateRequestNoir,
  TxContext as TxContextNoir,
  TxRequest as TxRequestNoir,
  ValidationRequests as ValidationRequestsNoir,
} from './types/index.js';

/* eslint-disable camelcase */

/**
 * Maps a field to a noir field.
 * @param field - The field.
 * @returns The noir field.
 */
export function mapFieldToNoir(field: Fr): NoirField {
  return field.toString();
}

/**
 * Maps a noir field to a fr.
 * @param field - The noir field.
 * @returns The fr.
 */
export function mapFieldFromNoir(field: NoirField): Fr {
  return Fr.fromString(field);
}

/** Maps a field to a noir wrapped field type (ie any type implemented as struct with an inner Field). */
export function mapWrappedFieldToNoir(field: Fr): { inner: NoirField } {
  return { inner: mapFieldToNoir(field) };
}

/** Maps a noir wrapped field type (ie any type implemented as struct with an inner Field) to a typescript field. */
export function mapWrappedFieldFromNoir(wrappedField: { inner: NoirField }): Fr {
  return mapFieldFromNoir(wrappedField.inner);
}

/**
 * Maps a number coming from noir.
 * @param number - The field representing the number.
 * @returns The number
 */
export function mapNumberFromNoir(number: NoirField): number {
  return Number(Fr.fromString(number).toBigInt());
}

export function mapNumberToNoir(number: number): NoirField {
  return new Fr(BigInt(number)).toString();
}

/**
 * Maps a point to a noir point.
 * @param point - The point.
 * @returns The noir point.
 */
export function mapPointToNoir(point: Point): NoirPoint {
  return {
    x: mapFieldToNoir(point.x),
    y: mapFieldToNoir(point.y),
    is_infinite: point.isInfinite,
  };
}

/**
 * Maps a noir point to a point.
 * @param point - The noir point.
 * @returns The point.
 */
export function mapPointFromNoir(point: NoirPoint): Point {
  return new Point(mapFieldFromNoir(point.x), mapFieldFromNoir(point.y), point.is_infinite);
}

/**
 * Maps a GrumpkinScalar to a noir GrumpkinScalar.
 * @param privateKey - The GrumpkinScalar.
 * @returns The noir GrumpkinScalar.
 */
export function mapGrumpkinScalarToNoir(privateKey: GrumpkinScalar): GrumpkinScalarNoir {
  return {
    hi: mapFieldToNoir(privateKey.hi),
    lo: mapFieldToNoir(privateKey.lo),
  };
}

/**
 * Maps a KeyValidationHint to noir.
 * @param hint - The key validation hint.
 * @returns The key validation hint mapped to noir types.
 */
export function mapKeyValidationHintToNoir(hint: KeyValidationHint): KeyValidationHintNoir {
  return {
    sk_m: mapGrumpkinScalarToNoir(hint.skM),
    request_index: mapNumberToNoir(hint.requestIndex),
  };
}

/**
 * Maps a noir GrumpkinScalar to a GrumpkinScalar.
 * @param privateKey - The noir GrumpkinScalar.
 * @returns The GrumpkinScalar.
 */
export function mapGrumpkinScalarFromNoir(privateKey: GrumpkinScalarNoir): GrumpkinScalar {
  return GrumpkinScalar.fromHighLow(mapFieldFromNoir(privateKey.hi), mapFieldFromNoir(privateKey.lo));
}

/**
 * Maps an aztec address to a noir aztec address.
 * @param address - The address.
 * @returns The noir aztec address.
 */
export function mapAztecAddressToNoir(address: AztecAddress): NoirAztecAddress {
  return {
    inner: mapFieldToNoir(address.toField()),
  };
}

/**
 * Maps a noir aztec address to an aztec address.
 * @param address - The noir aztec address.
 * @returns The aztec address.
 */
export function mapAztecAddressFromNoir(address: NoirAztecAddress): AztecAddress {
  return AztecAddress.fromField(mapFieldFromNoir(address.inner));
}

/**
 * Maps an eth address to a noir eth address.
 * @param address - The address.
 * @returns The noir eth address.
 */
export function mapEthAddressToNoir(address: EthAddress): NoirEthAddress {
  return {
    inner: mapFieldToNoir(address.toField()),
  };
}

/**
 * Maps a noir eth address to an eth address.
 * @param address - The noir eth address.
 * @returns The eth address.
 */
export function mapEthAddressFromNoir(address: NoirEthAddress): EthAddress {
  return EthAddress.fromField(mapFieldFromNoir(address.inner));
}

/**
 * Maps a tx context to a noir tx context.
 * @param txContext - The tx context.
 * @returns The noir tx context.
 */
export function mapTxContextToNoir(txContext: TxContext): TxContextNoir {
  return {
    chain_id: mapFieldToNoir(txContext.chainId),
    version: mapFieldToNoir(txContext.version),
    gas_settings: mapGasSettingsToNoir(txContext.gasSettings),
  };
}

/**
 * Maps a noir tx context to a tx context.
 * @param txContext - The noir tx context.
 * @returns The tx context.
 */
export function mapTxContextFromNoir(txContext: TxContextNoir): TxContext {
  return new TxContext(
    mapFieldFromNoir(txContext.chain_id),
    mapFieldFromNoir(txContext.version),
    mapGasSettingsFromNoir(txContext.gas_settings),
  );
}

/**
 * Maps a function selector to a noir function selector.
 * @param functionSelector - The function selector.
 * @returns The noir function selector.
 */
export function mapFunctionSelectorToNoir(functionSelector: FunctionSelector): FunctionSelectorNoir {
  return {
    inner: mapFieldToNoir(functionSelector.toField()),
  };
}

/**
 * Maps a noir function selector to a function selector.
 * @param functionSelector - The noir function selector.
 * @returns The function selector.
 */
export function mapFunctionSelectorFromNoir(functionSelector: FunctionSelectorNoir): FunctionSelector {
  return FunctionSelector.fromField(mapFieldFromNoir(functionSelector.inner));
}

/**
 * Maps a function data to a noir function data.
 * @param functionData - The function data.
 * @returns The noir function data.
 */
export function mapFunctionDataToNoir(functionData: FunctionData): FunctionDataNoir {
  return {
    selector: mapFunctionSelectorToNoir(functionData.selector),
    is_private: functionData.isPrivate,
  };
}

/**
 * Maps a noir function data to a function data.
 * @param functionData - The noir function data.
 * @returns The function data.
 */
export function mapFunctionDataFromNoir(functionData: FunctionDataNoir): FunctionData {
  return new FunctionData(mapFunctionSelectorFromNoir(functionData.selector), functionData.is_private);
}

/**
 * Maps a tx request to a noir tx request.
 * @param txRequest - The tx request.
 * @returns The noir tx request.
 */
export function mapTxRequestToNoir(txRequest: TxRequest): TxRequestNoir {
  return {
    origin: mapAztecAddressToNoir(txRequest.origin),
    args_hash: mapFieldToNoir(txRequest.argsHash),
    tx_context: mapTxContextToNoir(txRequest.txContext),
    function_data: mapFunctionDataToNoir(txRequest.functionData),
  };
}

/**
 * Maps a call context to a noir call context.
 * @param callContext - The call context.
 * @returns The noir call context.
 */
export function mapCallContextFromNoir(callContext: CallContextNoir): CallContext {
  return new CallContext(
    mapAztecAddressFromNoir(callContext.msg_sender),
    mapAztecAddressFromNoir(callContext.storage_contract_address),
    mapFunctionSelectorFromNoir(callContext.function_selector),
    callContext.is_delegate_call,
    callContext.is_static_call,
  );
}

/**
 * Maps a call context to a noir call context.
 * @param callContext - The call context.
 * @returns The noir call context.
 */
export function mapCallContextToNoir(callContext: CallContext): CallContextNoir {
  return {
    msg_sender: mapAztecAddressToNoir(callContext.msgSender),
    storage_contract_address: mapAztecAddressToNoir(callContext.storageContractAddress),
    function_selector: mapFunctionSelectorToNoir(callContext.functionSelector),
    is_delegate_call: callContext.isDelegateCall,
    is_static_call: callContext.isStaticCall,
  };
}

export function mapGasSettingsFromNoir(gasSettings: GasSettingsNoir): GasSettings {
  return new GasSettings(
    mapGasFromNoir(gasSettings.gas_limits),
    mapGasFromNoir(gasSettings.teardown_gas_limits),
    mapGasFeesFromNoir(gasSettings.max_fees_per_gas),
    mapFieldFromNoir(gasSettings.inclusion_fee),
  );
}

export function mapGasSettingsToNoir(gasSettings: GasSettings): GasSettingsNoir {
  return {
    gas_limits: mapGasToNoir(gasSettings.gasLimits),
    teardown_gas_limits: mapGasToNoir(gasSettings.teardownGasLimits),
    max_fees_per_gas: mapGasFeesToNoir(gasSettings.maxFeesPerGas),
    inclusion_fee: mapFieldToNoir(gasSettings.inclusionFee),
  };
}

/**
 * Maps a caller context to a noir caller context.
 * @param callContext - The caller context.
 * @returns The noir caller context.
 */
export function mapCallerContextFromNoir(callerContext: CallerContextNoir): CallerContext {
  return new CallerContext(
    mapAztecAddressFromNoir(callerContext.msg_sender),
    mapAztecAddressFromNoir(callerContext.storage_contract_address),
    callerContext.is_static_call,
  );
}

/**
 * Maps a caller context to a noir caller context.
 * @param callContext - The caller context.
 * @returns The noir caller context.
 */
export function mapCallerContextToNoir(callerContext: CallerContext): CallerContextNoir {
  return {
    msg_sender: mapAztecAddressToNoir(callerContext.msgSender),
    storage_contract_address: mapAztecAddressToNoir(callerContext.storageContractAddress),
    is_static_call: callerContext.isStaticCall,
  };
}

function mapPrivateCallRequestFromNoir(callRequest: PrivateCallRequestNoir) {
  return new PrivateCallRequest(
    mapAztecAddressFromNoir(callRequest.target),
    mapCallContextFromNoir(callRequest.call_context),
    mapFunctionDataFromNoir(callRequest.function_data),
    mapFieldFromNoir(callRequest.args_hash),
    mapFieldFromNoir(callRequest.returns_hash),
    mapCallerContextFromNoir(callRequest.caller_context),
    mapNumberFromNoir(callRequest.start_side_effect_counter),
    mapNumberFromNoir(callRequest.end_side_effect_counter),
  );
}

function mapPrivateCallRequestToNoir(callRequest: PrivateCallRequest): PrivateCallRequestNoir {
  return {
    target: mapAztecAddressToNoir(callRequest.target),
    call_context: mapCallContextToNoir(callRequest.callContext),
    function_data: mapFunctionDataToNoir(callRequest.functionData),
    args_hash: mapFieldToNoir(callRequest.argsHash),
    returns_hash: mapFieldToNoir(callRequest.returnsHash),
    caller_context: mapCallerContextToNoir(callRequest.callerContext),
    start_side_effect_counter: mapNumberToNoir(callRequest.startSideEffectCounter),
    end_side_effect_counter: mapNumberToNoir(callRequest.endSideEffectCounter),
  };
}

function mapScopedPrivateCallRequestFromNoir(callRequest: ScopedPrivateCallRequestNoir) {
  return new ScopedPrivateCallRequest(
    mapPrivateCallRequestFromNoir(callRequest.call_request),
    mapAztecAddressFromNoir(callRequest.contract_address),
  );
}

function mapScopedPrivateCallRequestToNoir(callRequest: ScopedPrivateCallRequest): ScopedPrivateCallRequestNoir {
  return {
    call_request: mapPrivateCallRequestToNoir(callRequest.callRequest),
    contract_address: mapAztecAddressToNoir(callRequest.contractAddress),
  };
}

/**
 * Maps a noir call request to a call request.
 * @param callRequest - The noir call request.
 * @returns The call request.
 */
export function mapCallRequestFromNoir(callRequest: CallRequestNoir): CallRequest {
  return new CallRequest(
    mapFieldFromNoir(callRequest.hash),
    mapAztecAddressFromNoir(callRequest.caller_contract_address),
    mapCallerContextFromNoir(callRequest.caller_context),
    mapFieldFromNoir(callRequest.start_side_effect_counter),
    mapFieldFromNoir(callRequest.end_side_effect_counter),
  );
}

/**
 * Maps a call request to a noir call request.
 * @param privateCallStackItem - The call stack item.
 * @returns The noir call stack item.
 */
export function mapCallRequestToNoir(callRequest: CallRequest): CallRequestNoir {
  return {
    hash: mapFieldToNoir(callRequest.hash),
    caller_contract_address: mapAztecAddressToNoir(callRequest.callerContractAddress),
    caller_context: mapCallerContextToNoir(callRequest.callerContext),
    start_side_effect_counter: mapFieldToNoir(callRequest.startSideEffectCounter),
    end_side_effect_counter: mapFieldToNoir(callRequest.endSideEffectCounter),
  };
}

function mapNoteHashToNoir(noteHash: NoteHash): NoteHashNoir {
  return {
    value: mapFieldToNoir(noteHash.value),
    counter: mapNumberToNoir(noteHash.counter),
  };
}

function mapNoteHashFromNoir(noteHash: NoteHashNoir) {
  return new NoteHash(mapFieldFromNoir(noteHash.value), mapNumberFromNoir(noteHash.counter));
}

function mapScopedNoteHashToNoir(noteHash: ScopedNoteHash): ScopedNoteHashNoir {
  return {
    note_hash: mapNoteHashToNoir(noteHash.noteHash),
    contract_address: mapAztecAddressToNoir(noteHash.contractAddress),
  };
}

function mapScopedNoteHashFromNoir(noteHash: ScopedNoteHashNoir) {
  return new ScopedNoteHash(
    mapNoteHashFromNoir(noteHash.note_hash),
    mapAztecAddressFromNoir(noteHash.contract_address),
  );
}

function mapNullifierToNoir(nullifier: Nullifier): NullifierNoir {
  return {
    value: mapFieldToNoir(nullifier.value),
    counter: mapNumberToNoir(nullifier.counter),
    note_hash: mapFieldToNoir(nullifier.noteHash),
  };
}

function mapNullifierFromNoir(nullifier: NullifierNoir) {
  return new Nullifier(
    mapFieldFromNoir(nullifier.value),
    mapNumberFromNoir(nullifier.counter),
    mapFieldFromNoir(nullifier.note_hash),
  );
}

function mapScopedNullifierToNoir(nullifier: ScopedNullifier): ScopedNullifierNoir {
  return {
    nullifier: mapNullifierToNoir(nullifier.nullifier),
    contract_address: mapAztecAddressToNoir(nullifier.contractAddress),
  };
}

function mapScopedNullifierFromNoir(nullifier: ScopedNullifierNoir) {
  return new ScopedNullifier(
    mapNullifierFromNoir(nullifier.nullifier),
    mapAztecAddressFromNoir(nullifier.contract_address),
  );
}

/**
 * Maps a LogHash to a noir LogHash.
 * @param logHash - The LogHash.
 * @returns The noir log hash.
 */
export function mapLogHashToNoir(logHash: LogHash): LogHashNoir {
  return {
    value: mapFieldToNoir(logHash.value),
    counter: mapNumberToNoir(logHash.counter),
    length: mapFieldToNoir(logHash.length),
  };
}

/**
 * Maps a noir LogHash to a LogHash.
 * @param logHash - The noir LogHash.
 * @returns The TS log hash.
 */
export function mapLogHashFromNoir(logHash: LogHashNoir): LogHash {
  return new LogHash(
    mapFieldFromNoir(logHash.value),
    mapNumberFromNoir(logHash.counter),
    mapFieldFromNoir(logHash.length),
  );
}

/**
 * Maps a LogHash to a noir LogHash.
 * @param logHash - The LogHash.
 * @returns The noir log hash.
 */
export function mapEncryptedLogHashToNoir(logHash: EncryptedLogHash): EncryptedLogHashNoir {
  return {
    value: mapFieldToNoir(logHash.value),
    counter: mapNumberToNoir(logHash.counter),
    length: mapFieldToNoir(logHash.length),
    randomness: mapFieldToNoir(logHash.randomness),
  };
}

/**
 * Maps a noir LogHash to a LogHash.
 * @param logHash - The noir LogHash.
 * @returns The TS log hash.
 */
export function mapEncryptedLogHashFromNoir(logHash: EncryptedLogHashNoir): EncryptedLogHash {
  return new EncryptedLogHash(
    mapFieldFromNoir(logHash.value),
    mapNumberFromNoir(logHash.counter),
    mapFieldFromNoir(logHash.length),
    mapFieldFromNoir(logHash.randomness),
  );
}

/**
 * Maps a ts ScopedLogHash to a noir ScopedLogHash.
 * @param logHash - The ts LogHash.
 * @returns The noir log hash.
 */
export function mapScopedEncryptedLogHashToNoir(scopedLogHash: ScopedEncryptedLogHash): ScopedEncryptedLogHashNoir {
  return {
    log_hash: mapEncryptedLogHashToNoir(scopedLogHash.logHash),
    contract_address: mapAztecAddressToNoir(scopedLogHash.contractAddress),
  };
}

/**
 * Maps a noir ScopedLogHash to a ts ScopedLogHash.
 * @param logHash - The noir LogHash.
 * @returns The TS log hash.
 */
export function mapScopedEncryptedLogHashFromNoir(scopedLogHash: ScopedEncryptedLogHashNoir): ScopedEncryptedLogHash {
  return new ScopedEncryptedLogHash(
    mapEncryptedLogHashFromNoir(scopedLogHash.log_hash),
    mapAztecAddressFromNoir(scopedLogHash.contract_address),
  );
}

/**
 * Maps a ts ScopedLogHash to a noir ScopedLogHash.
 * @param logHash - The ts LogHash.
 * @returns The noir log hash.
 */
export function mapScopedLogHashToNoir(scopedLogHash: ScopedLogHash): ScopedLogHashNoir {
  return {
    log_hash: mapLogHashToNoir(scopedLogHash.logHash),
    contract_address: mapAztecAddressToNoir(scopedLogHash.contractAddress),
  };
}

/**
 * Maps a noir ScopedLogHash to a ts ScopedLogHash.
 * @param logHash - The noir LogHash.
 * @returns The TS log hash.
 */
export function mapScopedLogHashFromNoir(scopedLogHash: ScopedLogHashNoir): ScopedLogHash {
  return new ScopedLogHash(
    mapLogHashFromNoir(scopedLogHash.log_hash),
    mapAztecAddressFromNoir(scopedLogHash.contract_address),
  );
}

/**
 * Maps a LogHash to a noir LogHash.
 * @param noteLogHash - The NoteLogHash.
 * @returns The noir note log hash.
 */
export function mapNoteLogHashToNoir(noteLogHash: NoteLogHash): NoteLogHashNoir {
  return {
    value: mapFieldToNoir(noteLogHash.value),
    counter: mapNumberToNoir(noteLogHash.counter),
    length: mapFieldToNoir(noteLogHash.length),
    note_hash_counter: mapNumberToNoir(noteLogHash.noteHashCounter),
  };
}

/**
 * Maps a noir LogHash to a LogHash.
 * @param noteLogHash - The noir NoteLogHash.
 * @returns The TS note log hash.
 */
export function mapNoteLogHashFromNoir(noteLogHash: NoteLogHashNoir): NoteLogHash {
  return new NoteLogHash(
    mapFieldFromNoir(noteLogHash.value),
    mapNumberFromNoir(noteLogHash.counter),
    mapFieldFromNoir(noteLogHash.length),
    mapNumberFromNoir(noteLogHash.note_hash_counter),
  );
}

/**
 * Maps a ReadRequest to a noir ReadRequest.
 * @param readRequest - The read request.
 * @returns The noir ReadRequest.
 */
export function mapReadRequestToNoir(readRequest: ReadRequest): ReadRequestNoir {
  return {
    value: mapFieldToNoir(readRequest.value),
    counter: mapNumberToNoir(readRequest.counter),
  };
}

/**
 * Maps a noir ReadRequest to ReadRequest.
 * @param readRequest - The noir ReadRequest.
 * @returns The TS ReadRequest.
 */
export function mapReadRequestFromNoir(readRequest: ReadRequestNoir): ReadRequest {
  return new ReadRequest(mapFieldFromNoir(readRequest.value), mapNumberFromNoir(readRequest.counter));
}

function mapScopedReadRequestToNoir(scopedReadRequest: ScopedReadRequest): ScopedReadRequestNoir {
  return {
    read_request: mapReadRequestToNoir(scopedReadRequest.readRequest),
    contract_address: mapAztecAddressToNoir(scopedReadRequest.contractAddress),
  };
}

/**
 * Maps a noir ReadRequest to ReadRequest.
 * @param readRequest - The noir ReadRequest.
 * @returns The TS ReadRequest.
 */
export function mapScopedReadRequestFromNoir(scoped: ScopedReadRequestNoir): ScopedReadRequest {
  return new ScopedReadRequest(
    mapReadRequestFromNoir(scoped.read_request),
    mapAztecAddressFromNoir(scoped.contract_address),
  );
}

/**
 * Maps a KeyValidationRequest to a noir KeyValidationRequest.
 * @param request - The KeyValidationRequest.
 * @returns The noir KeyValidationRequest.
 */
export function mapKeyValidationRequestToNoir(request: KeyValidationRequest): KeyValidationRequestsNoir {
  return {
    pk_m: mapPointToNoir(request.pkM),
    sk_app: mapFieldToNoir(request.skApp),
  };
}

export function mapKeyValidationRequestAndGeneratorToNoir(
  request: KeyValidationRequestAndGenerator,
): KeyValidationRequestAndGeneratorNoir {
  return {
    request: mapKeyValidationRequestToNoir(request.request),
    sk_app_generator: mapFieldToNoir(request.skAppGenerator),
  };
}

/**
 * Maps a noir KeyValidationRequest to KeyValidationRequest.
 * @param request - The noir KeyValidationRequest.
 * @returns The TS KeyValidationRequest.
 */
function mapKeyValidationRequestFromNoir(request: KeyValidationRequestsNoir): KeyValidationRequest {
  return new KeyValidationRequest(mapPointFromNoir(request.pk_m), mapFieldFromNoir(request.sk_app));
}

function mapKeyValidationRequestAndGeneratorFromNoir(
  request: KeyValidationRequestAndGeneratorNoir,
): KeyValidationRequestAndGenerator {
  return new KeyValidationRequestAndGenerator(
    mapKeyValidationRequestFromNoir(request.request),
    mapFieldFromNoir(request.sk_app_generator),
  );
}

function mapScopedKeyValidationRequestAndGeneratorToNoir(
  request: ScopedKeyValidationRequestAndGenerator,
): ScopedKeyValidationRequestAndGeneratorNoir {
  return {
    request: mapKeyValidationRequestAndGeneratorToNoir(request.request),
    contract_address: mapAztecAddressToNoir(request.contractAddress),
  };
}

function mapScopedKeyValidationRequestAndGeneratorFromNoir(
  request: ScopedKeyValidationRequestAndGeneratorNoir,
): ScopedKeyValidationRequestAndGenerator {
  return new ScopedKeyValidationRequestAndGenerator(
    mapKeyValidationRequestAndGeneratorFromNoir(request.request),
    mapAztecAddressFromNoir(request.contract_address),
  );
}

/**
 * Maps a L2 to L1 message to a noir L2 to L1 message.
 * @param message - The L2 to L1 message.
 * @returns The noir L2 to L1 message.
 */
export function mapL2ToL1MessageToNoir(message: L2ToL1Message): L2ToL1MessageNoir {
  return {
    recipient: mapEthAddressToNoir(message.recipient),
    content: mapFieldToNoir(message.content),
    counter: mapNumberToNoir(message.counter),
  };
}

function mapL2ToL1MessageFromNoir(message: L2ToL1MessageNoir) {
  return new L2ToL1Message(
    mapEthAddressFromNoir(message.recipient),
    mapFieldFromNoir(message.content),
    mapNumberFromNoir(message.counter),
  );
}

function mapScopedL2ToL1MessageFromNoir(message: ScopedL2ToL1MessageNoir) {
  return new ScopedL2ToL1Message(
    mapL2ToL1MessageFromNoir(message.message),
    mapAztecAddressFromNoir(message.contract_address),
  );
}

function mapScopedL2ToL1MessageToNoir(message: ScopedL2ToL1Message): ScopedL2ToL1MessageNoir {
  return {
    message: mapL2ToL1MessageToNoir(message.message),
    contract_address: mapAztecAddressToNoir(message.contractAddress),
  };
}

/**
 * Maps private circuit public inputs to noir private circuit public inputs.
 * @param privateCircuitPublicInputs - The private circuit public inputs.
 * @returns The noir private circuit public inputs.
 */
export function mapPrivateCircuitPublicInputsToNoir(
  privateCircuitPublicInputs: PrivateCircuitPublicInputs,
): PrivateCircuitPublicInputsNoir {
  return {
    max_block_number: mapMaxBlockNumberToNoir(privateCircuitPublicInputs.maxBlockNumber),
    call_context: mapCallContextToNoir(privateCircuitPublicInputs.callContext),
    args_hash: mapFieldToNoir(privateCircuitPublicInputs.argsHash),
    returns_hash: mapFieldToNoir(privateCircuitPublicInputs.returnsHash),
    note_hash_read_requests: mapTuple(privateCircuitPublicInputs.noteHashReadRequests, mapReadRequestToNoir),
    nullifier_read_requests: mapTuple(privateCircuitPublicInputs.nullifierReadRequests, mapReadRequestToNoir),
    key_validation_requests_and_generators: mapTuple(
      privateCircuitPublicInputs.keyValidationRequestsAndGenerators,
      mapKeyValidationRequestAndGeneratorToNoir,
    ),
    note_hashes: mapTuple(privateCircuitPublicInputs.noteHashes, mapNoteHashToNoir),
    nullifiers: mapTuple(privateCircuitPublicInputs.nullifiers, mapNullifierToNoir),
    private_call_requests: mapTuple(privateCircuitPublicInputs.privateCallRequests, mapPrivateCallRequestToNoir),
    public_call_stack_hashes: mapTuple(privateCircuitPublicInputs.publicCallStackHashes, mapFieldToNoir),
    public_teardown_function_hash: mapFieldToNoir(privateCircuitPublicInputs.publicTeardownFunctionHash),
    l2_to_l1_msgs: mapTuple(privateCircuitPublicInputs.l2ToL1Msgs, mapL2ToL1MessageToNoir),
    start_side_effect_counter: mapFieldToNoir(privateCircuitPublicInputs.startSideEffectCounter),
    end_side_effect_counter: mapFieldToNoir(privateCircuitPublicInputs.endSideEffectCounter),
    note_encrypted_logs_hashes: mapTuple(privateCircuitPublicInputs.noteEncryptedLogsHashes, mapNoteLogHashToNoir),
    encrypted_logs_hashes: mapTuple(privateCircuitPublicInputs.encryptedLogsHashes, mapEncryptedLogHashToNoir),
    unencrypted_logs_hashes: mapTuple(privateCircuitPublicInputs.unencryptedLogsHashes, mapLogHashToNoir),
    historical_header: mapHeaderToNoir(privateCircuitPublicInputs.historicalHeader),
    tx_context: mapTxContextToNoir(privateCircuitPublicInputs.txContext),
    min_revertible_side_effect_counter: mapFieldToNoir(privateCircuitPublicInputs.minRevertibleSideEffectCounter),
    is_fee_payer: privateCircuitPublicInputs.isFeePayer,
  };
}

/**
 * Maps a private call stack item to a noir private call stack item.
 * @param privateCallStackItem - The private call stack item.
 * @returns The noir private call stack item.
 */
export function mapPrivateCallStackItemToNoir(privateCallStackItem: PrivateCallStackItem): PrivateCallStackItemNoir {
  return {
    contract_address: mapAztecAddressToNoir(privateCallStackItem.contractAddress),
    function_data: mapFunctionDataToNoir(privateCallStackItem.functionData),
    public_inputs: mapPrivateCircuitPublicInputsToNoir(privateCallStackItem.publicInputs),
  };
}

/**
 * Maps a private call data to a noir private call data.
 * @param privateCallData - The private call data.
 * @returns The noir private call data.
 */
export function mapPrivateCallDataToNoir(privateCallData: PrivateCallData): PrivateCallDataNoir {
  return {
    call_stack_item: mapPrivateCallStackItemToNoir(privateCallData.callStackItem),
    public_call_stack: mapTuple(privateCallData.publicCallStack, mapCallRequestToNoir),
    public_teardown_call_request: mapCallRequestToNoir(privateCallData.publicTeardownCallRequest),
    vk: mapVerificationKeyToNoir(privateCallData.vk),
    function_leaf_membership_witness: mapMembershipWitnessToNoir(privateCallData.functionLeafMembershipWitness),
    contract_class_artifact_hash: mapFieldToNoir(privateCallData.contractClassArtifactHash),
    contract_class_public_bytecode_commitment: mapFieldToNoir(privateCallData.contractClassPublicBytecodeCommitment),
    public_keys_hash: mapWrappedFieldToNoir(privateCallData.publicKeysHash),
    salted_initialization_hash: mapWrappedFieldToNoir(privateCallData.saltedInitializationHash),
    acir_hash: mapFieldToNoir(privateCallData.acirHash),
  };
}

export function mapRevertCodeFromNoir(revertCode: NoirField): RevertCode {
  return RevertCode.fromField(mapFieldFromNoir(revertCode));
}

export function mapRevertCodeToNoir(revertCode: RevertCode): NoirField {
  return mapFieldToNoir(revertCode.toField());
}

/**
 * Maps an array from noir types to a tuple of parsed types.
 * @param noirArray - The noir array.
 * @param length - The length of the tuple.
 * @param mapper - The mapper function applied to each element.
 * @returns The tuple.
 */
export function mapTupleFromNoir<T, N extends number, M>(
  noirArray: T[],
  length: N,
  mapper: (item: T) => M,
): Tuple<M, N> {
  if (noirArray.length != length) {
    throw new Error(`Expected ${length} items, got ${noirArray.length}`);
  }
  return Array.from({ length }, (_, idx) => mapper(noirArray[idx])) as Tuple<M, N>;
}

/**
 * Maps a SHA256 hash from noir to the parsed type.
 * @param hash - The hash as it is represented in Noir (1 fields).
 * @returns The hash represented as a 31 bytes long buffer.
 */
export function mapSha256HashFromNoir(hash: Field): Buffer {
  return toBufferBE(mapFieldFromNoir(hash).toBigInt(), NUM_BYTES_PER_SHA256);
}

/**
 * Maps a sha256 to the representation used in noir.
 * @param hash - The hash represented as a 32 bytes long buffer.
 * @returns The hash as it is represented in Noir (1 field, truncated).
 */
export function mapSha256HashToNoir(hash: Buffer): Field {
  return mapFieldToNoir(toTruncField(hash));
}

/**
 * Maps public data update request from noir to the parsed type.
 * @param publicDataUpdateRequest - The noir public data update request.
 * @returns The parsed public data update request.
 */
export function mapPublicDataUpdateRequestFromNoir(
  publicDataUpdateRequest: PublicDataUpdateRequestNoir,
): PublicDataUpdateRequest {
  return new PublicDataUpdateRequest(
    mapFieldFromNoir(publicDataUpdateRequest.leaf_slot),
    mapFieldFromNoir(publicDataUpdateRequest.new_value),
    mapNumberFromNoir(publicDataUpdateRequest.counter),
  );
}

/**
 * Maps public data update request to noir public data update request.
 * @param publicDataUpdateRequest - The public data update request.
 * @returns The noir public data update request.
 */
export function mapPublicDataUpdateRequestToNoir(
  publicDataUpdateRequest: PublicDataUpdateRequest,
): PublicDataUpdateRequestNoir {
  return {
    leaf_slot: mapFieldToNoir(publicDataUpdateRequest.leafSlot),
    new_value: mapFieldToNoir(publicDataUpdateRequest.newValue),
    counter: mapNumberToNoir(publicDataUpdateRequest.sideEffectCounter),
  };
}

/**
 * Maps public data read from noir to the parsed type.
 * @param publicDataRead - The noir public data read.
 * @returns The parsed public data read.
 */
export function mapPublicDataReadFromNoir(publicDataRead: PublicDataReadNoir): PublicDataRead {
  return new PublicDataRead(mapFieldFromNoir(publicDataRead.leaf_slot), mapFieldFromNoir(publicDataRead.value));
}

/**
 * Maps public data read to noir public data read.
 * @param publicDataRead - The public data read.
 * @returns The noir public data read.
 */
export function mapPublicDataReadToNoir(publicDataRead: PublicDataRead): PublicDataReadNoir {
  return {
    leaf_slot: mapFieldToNoir(publicDataRead.leafSlot),
    value: mapFieldToNoir(publicDataRead.value),
  };
}

function mapReadRequestStatusToNoir(readRequestStatus: ReadRequestStatus): ReadRequestStatusNoir {
  return {
    state: mapNumberToNoir(readRequestStatus.state),
    hint_index: mapNumberToNoir(readRequestStatus.hintIndex),
  };
}

function mapPendingReadHintToNoir(hint: PendingReadHint): PendingReadHintNoir {
  return {
    read_request_index: mapNumberToNoir(hint.readRequestIndex),
    pending_value_index: mapNumberToNoir(hint.pendingValueIndex),
  };
}

function mapLeafDataReadHintToNoir(hint: LeafDataReadHint): LeafDataReadHintNoir {
  return {
    read_request_index: mapNumberToNoir(hint.readRequestIndex),
    data_hint_index: mapNumberToNoir(hint.dataHintIndex),
  };
}

function mapNoteHashSettledReadHintToNoir(
  hint: SettledReadHint<typeof NOTE_HASH_TREE_HEIGHT, Fr>,
): NoteHashSettledReadHintNoir {
  return {
    read_request_index: mapNumberToNoir(hint.readRequestIndex),
    membership_witness: mapMembershipWitnessToNoir(hint.membershipWitness),
    leaf_preimage: mapNoteHashLeafPreimageToNoir(hint.leafPreimage),
  };
}

function mapNullifierSettledReadHintToNoir(
  hint: SettledReadHint<typeof NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage>,
): NullifierSettledReadHintNoir {
  return {
    read_request_index: mapNumberToNoir(hint.readRequestIndex),
    membership_witness: mapMembershipWitnessToNoir(hint.membershipWitness),
    leaf_preimage: mapNullifierLeafPreimageToNoir(hint.leafPreimage),
  };
}

function mapNoteHashReadRequestHintsToNoir<PENDING extends number, SETTLED extends number>(
  hints: NoteHashReadRequestHints<PENDING, SETTLED>,
): NoteHashReadRequestHintsNoir<PENDING, SETTLED> {
  return {
    read_request_statuses: mapTuple(hints.readRequestStatuses, mapReadRequestStatusToNoir),
    pending_read_hints: hints.pendingReadHints.map(mapPendingReadHintToNoir) as FixedLengthArray<
      PendingReadHintNoir,
      PENDING
    >,
    settled_read_hints: hints.settledReadHints.map(mapNoteHashSettledReadHintToNoir) as FixedLengthArray<
      NoteHashSettledReadHintNoir,
      SETTLED
    >,
  };
}

function mapNullifierReadRequestHintsToNoir<PENDING extends number, SETTLED extends number>(
  hints: NullifierReadRequestHints<PENDING, SETTLED>,
): NullifierReadRequestHintsNoir<PENDING, SETTLED> {
  return {
    read_request_statuses: mapTuple(hints.readRequestStatuses, mapReadRequestStatusToNoir),
    pending_read_hints: hints.pendingReadHints.map(mapPendingReadHintToNoir) as FixedLengthArray<
      PendingReadHintNoir,
      PENDING
    >,
    settled_read_hints: hints.settledReadHints.map(settledHint =>
      mapNullifierSettledReadHintToNoir(
        settledHint as SettledReadHint<typeof NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage>,
      ),
    ) as FixedLengthArray<NullifierSettledReadHintNoir, SETTLED>,
  };
}

function mapNullifierNonMembershipHintToNoir(
  hint: NonMembershipHint<typeof NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage>,
): NullifierNonMembershipHintNoir {
  return {
    low_leaf_preimage: mapNullifierLeafPreimageToNoir(hint.leafPreimage),
    membership_witness: mapMembershipWitnessToNoir(hint.membershipWitness),
  };
}

function mapNullifierNonExistentReadRequestHintsToNoir(
  hints: NullifierNonExistentReadRequestHints,
): NullifierNonExistentReadRequestHintsNoir {
  return {
    non_membership_hints: mapTuple(hints.nonMembershipHints, mapNullifierNonMembershipHintToNoir),
    sorted_pending_values: mapTuple(hints.sortedPendingValues, mapNullifierToNoir),
    sorted_pending_value_index_hints: mapTuple(hints.sortedPendingValueHints, mapNumberToNoir),
    next_pending_value_indices: mapTuple(hints.nextPendingValueIndices, mapNumberToNoir),
  };
}

function mapPublicDataHintToNoir(hint: PublicDataHint): PublicDataHintNoir {
  return {
    leaf_slot: mapFieldToNoir(hint.leafSlot),
    value: mapFieldToNoir(hint.value),
    override_counter: mapNumberToNoir(hint.overrideCounter),
    membership_witness: mapMembershipWitnessToNoir(hint.membershipWitness),
    leaf_preimage: mapPublicDataTreePreimageToNoir(hint.leafPreimage),
  };
}

function mapPublicDataReadRequestHintsToNoir(hints: PublicDataReadRequestHints): PublicDataReadRequestHintsNoir {
  return {
    read_request_statuses: mapTuple(hints.readRequestStatuses, mapReadRequestStatusToNoir),
    pending_read_hints: mapTuple(hints.pendingReadHints, mapPendingReadHintToNoir),
    leaf_data_read_hints: mapTuple(hints.leafDataReadHints, mapLeafDataReadHintToNoir),
  };
}

function mapValidationRequestsToNoir(requests: ValidationRequests): ValidationRequestsNoir {
  return {
    for_rollup: mapRollupValidationRequestsToNoir(requests.forRollup),
    note_hash_read_requests: mapTuple(requests.noteHashReadRequests, mapScopedReadRequestToNoir),
    nullifier_read_requests: mapTuple(requests.nullifierReadRequests, mapScopedReadRequestToNoir),
    nullifier_non_existent_read_requests: mapTuple(
      requests.nullifierNonExistentReadRequests,
      mapScopedReadRequestToNoir,
    ),
    scoped_key_validation_requests_and_generators: mapTuple(
      requests.scopedKeyValidationRequestsAndGenerators,
      mapScopedKeyValidationRequestAndGeneratorToNoir,
    ),
    public_data_reads: mapTuple(requests.publicDataReads, mapPublicDataReadToNoir),
  };
}

function mapValidationRequestsFromNoir(requests: ValidationRequestsNoir): ValidationRequests {
  return new ValidationRequests(
    mapRollupValidationRequestsFromNoir(requests.for_rollup),
    mapTupleFromNoir(
      requests.note_hash_read_requests,
      MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
      mapScopedReadRequestFromNoir,
    ),
    mapTupleFromNoir(
      requests.nullifier_read_requests,
      MAX_NULLIFIER_READ_REQUESTS_PER_TX,
      mapScopedReadRequestFromNoir,
    ),
    mapTupleFromNoir(
      requests.nullifier_non_existent_read_requests,
      MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
      mapScopedReadRequestFromNoir,
    ),
    mapTupleFromNoir(
      requests.scoped_key_validation_requests_and_generators,
      MAX_KEY_VALIDATION_REQUESTS_PER_TX,
      mapScopedKeyValidationRequestAndGeneratorFromNoir,
    ),
    mapTupleFromNoir(requests.public_data_reads, MAX_PUBLIC_DATA_READS_PER_TX, mapPublicDataReadFromNoir),
  );
}

export function mapPrivateAccumulatedDataFromNoir(
  privateAccumulatedData: PrivateAccumulatedDataNoir,
): PrivateAccumulatedData {
  return new PrivateAccumulatedData(
    mapTupleFromNoir(privateAccumulatedData.note_hashes, MAX_NOTE_HASHES_PER_TX, mapScopedNoteHashFromNoir),
    mapTupleFromNoir(privateAccumulatedData.nullifiers, MAX_NULLIFIERS_PER_TX, mapScopedNullifierFromNoir),
    mapTupleFromNoir(privateAccumulatedData.l2_to_l1_msgs, MAX_L2_TO_L1_MSGS_PER_TX, mapScopedL2ToL1MessageFromNoir),
    mapTupleFromNoir(
      privateAccumulatedData.note_encrypted_logs_hashes,
      MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
      mapNoteLogHashFromNoir,
    ),
    mapTupleFromNoir(
      privateAccumulatedData.encrypted_logs_hashes,
      MAX_ENCRYPTED_LOGS_PER_TX,
      mapScopedEncryptedLogHashFromNoir,
    ),
    mapTupleFromNoir(
      privateAccumulatedData.unencrypted_logs_hashes,
      MAX_UNENCRYPTED_LOGS_PER_TX,
      mapScopedLogHashFromNoir,
    ),
    mapTupleFromNoir(
      privateAccumulatedData.private_call_stack,
      MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
      mapScopedPrivateCallRequestFromNoir,
    ),
    mapTupleFromNoir(
      privateAccumulatedData.public_call_stack,
      MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      mapCallRequestFromNoir,
    ),
  );
}

export function mapPrivateAccumulatedDataToNoir(data: PrivateAccumulatedData): PrivateAccumulatedDataNoir {
  return {
    note_hashes: mapTuple(data.noteHashes, mapScopedNoteHashToNoir),
    nullifiers: mapTuple(data.nullifiers, mapScopedNullifierToNoir),
    l2_to_l1_msgs: mapTuple(data.l2ToL1Msgs, mapScopedL2ToL1MessageToNoir),
    note_encrypted_logs_hashes: mapTuple(data.noteEncryptedLogsHashes, mapNoteLogHashToNoir),
    encrypted_logs_hashes: mapTuple(data.encryptedLogsHashes, mapScopedEncryptedLogHashToNoir),
    unencrypted_logs_hashes: mapTuple(data.unencryptedLogsHashes, mapScopedLogHashToNoir),
    private_call_stack: mapTuple(data.privateCallStack, mapScopedPrivateCallRequestToNoir),
    public_call_stack: mapTuple(data.publicCallStack, mapCallRequestToNoir),
  };
}

export function mapPublicAccumulatedDataFromNoir(
  publicAccumulatedData: PublicAccumulatedDataNoir,
): PublicAccumulatedData {
  return new PublicAccumulatedData(
    mapTupleFromNoir(publicAccumulatedData.note_hashes, MAX_NOTE_HASHES_PER_TX, mapNoteHashFromNoir),
    mapTupleFromNoir(publicAccumulatedData.nullifiers, MAX_NULLIFIERS_PER_TX, mapNullifierFromNoir),
    mapTupleFromNoir(publicAccumulatedData.l2_to_l1_msgs, MAX_L2_TO_L1_MSGS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(
      publicAccumulatedData.note_encrypted_logs_hashes,
      MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
      mapLogHashFromNoir,
    ),
    mapTupleFromNoir(publicAccumulatedData.encrypted_logs_hashes, MAX_ENCRYPTED_LOGS_PER_TX, mapLogHashFromNoir),
    mapTupleFromNoir(
      publicAccumulatedData.unencrypted_logs_hashes,
      MAX_UNENCRYPTED_LOGS_PER_TX,
      mapScopedLogHashFromNoir,
    ),
    mapTupleFromNoir(
      publicAccumulatedData.public_data_update_requests,
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      mapPublicDataUpdateRequestFromNoir,
    ),
    mapTupleFromNoir(
      publicAccumulatedData.public_call_stack,
      MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      mapCallRequestFromNoir,
    ),
    mapGasFromNoir(publicAccumulatedData.gas_used),
  );
}

export function mapPublicAccumulatedDataToNoir(
  publicAccumulatedData: PublicAccumulatedData,
): PublicAccumulatedDataNoir {
  return {
    note_hashes: mapTuple(publicAccumulatedData.noteHashes, mapNoteHashToNoir),
    nullifiers: mapTuple(publicAccumulatedData.nullifiers, mapNullifierToNoir),
    l2_to_l1_msgs: mapTuple(publicAccumulatedData.l2ToL1Msgs, mapFieldToNoir),
    note_encrypted_logs_hashes: mapTuple(publicAccumulatedData.noteEncryptedLogsHashes, mapLogHashToNoir),
    encrypted_logs_hashes: mapTuple(publicAccumulatedData.encryptedLogsHashes, mapLogHashToNoir),
    unencrypted_logs_hashes: mapTuple(publicAccumulatedData.unencryptedLogsHashes, mapScopedLogHashToNoir),
    public_data_update_requests: mapTuple(
      publicAccumulatedData.publicDataUpdateRequests,
      mapPublicDataUpdateRequestToNoir,
    ),
    public_call_stack: mapTuple(publicAccumulatedData.publicCallStack, mapCallRequestToNoir),
    gas_used: mapGasToNoir(publicAccumulatedData.gasUsed),
  };
}

export function mapGasFromNoir(gasUsed: GasNoir): Gas {
  return Gas.from({
    daGas: mapNumberFromNoir(gasUsed.da_gas),
    l2Gas: mapNumberFromNoir(gasUsed.l2_gas),
  });
}

export function mapGasToNoir(gasUsed: Gas): GasNoir {
  return {
    da_gas: mapNumberToNoir(gasUsed.daGas),
    l2_gas: mapNumberToNoir(gasUsed.l2Gas),
  };
}

export function mapRollupValidationRequestsToNoir(
  rollupValidationRequests: RollupValidationRequests,
): RollupValidationRequestsNoir {
  return {
    max_block_number: mapMaxBlockNumberToNoir(rollupValidationRequests.maxBlockNumber),
  };
}

export function mapRollupValidationRequestsFromNoir(
  rollupValidationRequests: RollupValidationRequestsNoir,
): RollupValidationRequests {
  return new RollupValidationRequests(mapMaxBlockNumberFromNoir(rollupValidationRequests.max_block_number));
}

export function mapMaxBlockNumberToNoir(maxBlockNumber: MaxBlockNumber): MaxBlockNumberNoir {
  return {
    _opt: {
      _is_some: maxBlockNumber.isSome,
      _value: mapFieldToNoir(maxBlockNumber.value),
    },
  };
}

export function mapMaxBlockNumberFromNoir(maxBlockNumber: MaxBlockNumberNoir): MaxBlockNumber {
  return new MaxBlockNumber(maxBlockNumber._opt._is_some, mapFieldFromNoir(maxBlockNumber._opt._value));
}

/**
 * Maps combined accumulated data from noir to the parsed type.
 * @param combinedAccumulatedData - The noir combined accumulated data.
 * @returns The parsed combined accumulated data.
 */
export function mapCombinedAccumulatedDataFromNoir(
  combinedAccumulatedData: CombinedAccumulatedDataNoir,
): CombinedAccumulatedData {
  return new CombinedAccumulatedData(
    mapTupleFromNoir(combinedAccumulatedData.note_hashes, MAX_NOTE_HASHES_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.nullifiers, MAX_NULLIFIERS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.l2_to_l1_msgs, MAX_L2_TO_L1_MSGS_PER_TX, mapFieldFromNoir),
    mapFieldFromNoir(combinedAccumulatedData.note_encrypted_logs_hash),
    mapFieldFromNoir(combinedAccumulatedData.encrypted_logs_hash),
    mapTupleFromNoir(
      combinedAccumulatedData.unencrypted_logs_hashes,
      MAX_UNENCRYPTED_LOGS_PER_TX,
      mapScopedLogHashFromNoir,
    ),
    mapFieldFromNoir(combinedAccumulatedData.note_encrypted_log_preimages_length),
    mapFieldFromNoir(combinedAccumulatedData.encrypted_log_preimages_length),
    mapFieldFromNoir(combinedAccumulatedData.unencrypted_log_preimages_length),
    mapTupleFromNoir(
      combinedAccumulatedData.public_data_update_requests,
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      mapPublicDataUpdateRequestFromNoir,
    ),
    mapGasFromNoir(combinedAccumulatedData.gas_used),
  );
}

export function mapCombinedAccumulatedDataToNoir(
  combinedAccumulatedData: CombinedAccumulatedData,
): CombinedAccumulatedDataNoir {
  return {
    note_hashes: mapTuple(combinedAccumulatedData.noteHashes, mapFieldToNoir),
    nullifiers: mapTuple(combinedAccumulatedData.nullifiers, mapFieldToNoir),
    l2_to_l1_msgs: mapTuple(combinedAccumulatedData.l2ToL1Msgs, mapFieldToNoir),
    note_encrypted_logs_hash: mapFieldToNoir(combinedAccumulatedData.noteEncryptedLogsHash),
    encrypted_logs_hash: mapFieldToNoir(combinedAccumulatedData.encryptedLogsHash),
    unencrypted_logs_hashes: mapTuple(combinedAccumulatedData.unencryptedLogsHashes, mapScopedLogHashToNoir),
    note_encrypted_log_preimages_length: mapFieldToNoir(combinedAccumulatedData.noteEncryptedLogPreimagesLength),
    encrypted_log_preimages_length: mapFieldToNoir(combinedAccumulatedData.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: mapFieldToNoir(combinedAccumulatedData.unencryptedLogPreimagesLength),
    public_data_update_requests: mapTuple(
      combinedAccumulatedData.publicDataUpdateRequests,
      mapPublicDataUpdateRequestToNoir,
    ),
    gas_used: mapGasToNoir(combinedAccumulatedData.gasUsed),
  };
}

/**
 * Maps combined constant data from noir to the parsed type.
 * @param combinedConstantData - The noir combined constant data.
 * @returns The parsed combined constant data.
 */
export function mapCombinedConstantDataFromNoir(combinedConstantData: CombinedConstantDataNoir): CombinedConstantData {
  return new CombinedConstantData(
    mapHeaderFromNoir(combinedConstantData.historical_header),
    mapTxContextFromNoir(combinedConstantData.tx_context),
    mapFieldFromNoir(combinedConstantData.vk_tree_root),
    mapGlobalVariablesFromNoir(combinedConstantData.global_variables),
  );
}

/**
 * Maps combined constant data to noir combined constant data.
 * @param combinedConstantData - The combined constant data.
 * @returns The noir combined constant data.
 */
export function mapCombinedConstantDataToNoir(combinedConstantData: CombinedConstantData): CombinedConstantDataNoir {
  return {
    historical_header: mapHeaderToNoir(combinedConstantData.historicalHeader),
    tx_context: mapTxContextToNoir(combinedConstantData.txContext),
    vk_tree_root: mapFieldToNoir(combinedConstantData.vkTreeRoot),
    global_variables: mapGlobalVariablesToNoir(combinedConstantData.globalVariables),
  };
}

export function mapPublicKernelCircuitPublicInputsToNoir(
  inputs: PublicKernelCircuitPublicInputs,
): PublicKernelCircuitPublicInputsNoir {
  return {
    constants: mapCombinedConstantDataToNoir(inputs.constants),
    validation_requests: mapValidationRequestsToNoir(inputs.validationRequests),
    end: mapPublicAccumulatedDataToNoir(inputs.end),
    end_non_revertible: mapPublicAccumulatedDataToNoir(inputs.endNonRevertibleData),
    revert_code: mapRevertCodeToNoir(inputs.revertCode),
    public_teardown_call_stack: mapTuple(inputs.publicTeardownCallStack, mapCallRequestToNoir),
    fee_payer: mapAztecAddressToNoir(inputs.feePayer),
  };
}

export function mapKernelCircuitPublicInputsFromNoir(inputs: KernelCircuitPublicInputsNoir) {
  return new KernelCircuitPublicInputs(
    mapRollupValidationRequestsFromNoir(inputs.rollup_validation_requests),
    mapCombinedAccumulatedDataFromNoir(inputs.end),
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapPartialStateReferenceFromNoir(inputs.start_state),
    mapRevertCodeFromNoir(inputs.revert_code),
    mapAztecAddressFromNoir(inputs.fee_payer),
  );
}

export function mapKernelCircuitPublicInputsToNoir(inputs: KernelCircuitPublicInputs): KernelCircuitPublicInputsNoir {
  return {
    rollup_validation_requests: mapRollupValidationRequestsToNoir(inputs.rollupValidationRequests),
    constants: mapCombinedConstantDataToNoir(inputs.constants),
    end: mapCombinedAccumulatedDataToNoir(inputs.end),
    start_state: mapPartialStateReferenceToNoir(inputs.startState),
    revert_code: mapRevertCodeToNoir(inputs.revertCode),
    fee_payer: mapAztecAddressToNoir(inputs.feePayer),
  };
}

/**
 * Maps a public kernel inner data to a noir public kernel data.
 * @param publicKernelData - The public kernel inner data.
 * @returns The noir public kernel data.
 */
export function mapPublicKernelDataToNoir(publicKernelData: PublicKernelData): PublicKernelDataNoir {
  return {
    public_inputs: mapPublicKernelCircuitPublicInputsToNoir(publicKernelData.publicInputs),
    proof: mapRecursiveProofToNoir<typeof NESTED_RECURSIVE_PROOF_LENGTH>(publicKernelData.proof),
    vk: mapVerificationKeyToNoir(publicKernelData.vk.keyAsFields),
    vk_index: mapFieldToNoir(new Fr(publicKernelData.vkIndex)),
    vk_path: mapTuple(publicKernelData.vkPath, mapFieldToNoir),
  };
}

export function mapKernelDataToNoir(kernelData: KernelData): KernelDataNoir {
  return {
    public_inputs: mapKernelCircuitPublicInputsToNoir(kernelData.publicInputs),
    proof: mapRecursiveProofToNoir<typeof NESTED_RECURSIVE_PROOF_LENGTH>(kernelData.proof),
    vk: mapVerificationKeyToNoir(kernelData.vk.keyAsFields),
    vk_index: mapFieldToNoir(new Fr(kernelData.vkIndex)),
    vk_path: mapTuple(kernelData.vkPath, mapFieldToNoir),
  };
}

export function mapVerificationKeyToNoir(key: VerificationKeyAsFields) {
  return {
    key: mapTuple(key.key, mapFieldToNoir),
    hash: mapFieldToNoir(key.hash),
  };
}

export function mapPrivateKernelCircuitPublicInputsFromNoir(
  inputs: PrivateKernelCircuitPublicInputsNoir,
): PrivateKernelCircuitPublicInputs {
  return new PrivateKernelCircuitPublicInputs(
    mapFieldFromNoir(inputs.min_revertible_side_effect_counter),
    mapValidationRequestsFromNoir(inputs.validation_requests),
    mapPrivateAccumulatedDataFromNoir(inputs.end),
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapCallRequestFromNoir(inputs.public_teardown_call_request),
    mapAztecAddressFromNoir(inputs.fee_payer),
  );
}

export function mapPrivateKernelCircuitPublicInputsToNoir(
  inputs: PrivateKernelCircuitPublicInputs,
): PrivateKernelCircuitPublicInputsNoir {
  return {
    constants: mapCombinedConstantDataToNoir(inputs.constants),
    validation_requests: mapValidationRequestsToNoir(inputs.validationRequests),
    end: mapPrivateAccumulatedDataToNoir(inputs.end),
    min_revertible_side_effect_counter: mapFieldToNoir(inputs.minRevertibleSideEffectCounter),
    public_teardown_call_request: mapCallRequestToNoir(inputs.publicTeardownCallRequest),
    fee_payer: mapAztecAddressToNoir(inputs.feePayer),
  };
}

/**
 * Maps a private kernel inner data to a noir private kernel inner data.
 * @param privateKernelInnerData - The private kernel inner data.
 * @returns The noir private kernel inner data.
 */
export function mapPrivateKernelDataToNoir(privateKernelInnerData: PrivateKernelData): PrivateKernelDataNoir {
  return {
    public_inputs: mapPrivateKernelCircuitPublicInputsToNoir(privateKernelInnerData.publicInputs),
    vk: mapVerificationKeyToNoir(privateKernelInnerData.vk),
    vk_index: mapFieldToNoir(new Fr(privateKernelInnerData.vkIndex)),
    vk_path: mapTuple(privateKernelInnerData.vkPath, mapFieldToNoir),
  };
}

export function mapPrivateKernelTailCircuitPublicInputsForRollupFromNoir(
  inputs: KernelCircuitPublicInputsNoir,
): PrivateKernelTailCircuitPublicInputs {
  const forRollup = new PartialPrivateTailPublicInputsForRollup(
    mapRollupValidationRequestsFromNoir(inputs.rollup_validation_requests),
    mapCombinedAccumulatedDataFromNoir(inputs.end),
  );
  return new PrivateKernelTailCircuitPublicInputs(
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapRevertCodeFromNoir(inputs.revert_code),
    mapAztecAddressFromNoir(inputs.fee_payer),
    undefined,
    forRollup,
  );
}

export function mapPrivateKernelTailCircuitPublicInputsForPublicFromNoir(
  inputs: PublicKernelCircuitPublicInputsNoir,
): PrivateKernelTailCircuitPublicInputs {
  const forPublic = new PartialPrivateTailPublicInputsForPublic(
    mapValidationRequestsFromNoir(inputs.validation_requests),
    mapPublicAccumulatedDataFromNoir(inputs.end_non_revertible),
    mapPublicAccumulatedDataFromNoir(inputs.end),
    mapTupleFromNoir(inputs.public_teardown_call_stack, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, mapCallRequestFromNoir),
  );
  return new PrivateKernelTailCircuitPublicInputs(
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapRevertCodeFromNoir(inputs.revert_code),
    mapAztecAddressFromNoir(inputs.fee_payer),
    forPublic,
  );
}

export function mapPrivateKernelInitCircuitPrivateInputsToNoir(
  inputs: PrivateKernelInitCircuitPrivateInputs,
): PrivateKernelInitCircuitPrivateInputsNoir {
  return {
    tx_request: mapTxRequestToNoir(inputs.txRequest),
    private_call: mapPrivateCallDataToNoir(inputs.privateCall),
    vk_tree_root: mapFieldToNoir(inputs.vkTreeRoot),
  };
}

export function mapPrivateKernelInnerCircuitPrivateInputsToNoir(
  inputs: PrivateKernelInnerCircuitPrivateInputs,
): PrivateKernelInnerCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPrivateKernelDataToNoir(inputs.previousKernel),
    private_call: mapPrivateCallDataToNoir(inputs.privateCall),
  };
}

function mapPrivateKernelResetHintsToNoir<
  NH_RR_PENDING extends number,
  NH_RR_SETTLED extends number,
  NLL_RR_PENDING extends number,
  NLL_RR_SETTLED extends number,
  KEY_VALIDATION_REQUESTS extends number,
>(
  inputs: PrivateKernelResetHints<
    NH_RR_PENDING,
    NH_RR_SETTLED,
    NLL_RR_PENDING,
    NLL_RR_SETTLED,
    KEY_VALIDATION_REQUESTS
  >,
): PrivateKernelResetHintsNoir<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> {
  return {
    transient_nullifier_indexes_for_note_hashes: mapTuple(
      inputs.transientNullifierIndexesForNoteHashes,
      mapNumberToNoir,
    ),
    transient_note_hash_indexes_for_nullifiers: mapTuple(inputs.transientNoteHashIndexesForNullifiers, mapNumberToNoir),
    note_hash_read_request_hints: mapNoteHashReadRequestHintsToNoir(inputs.noteHashReadRequestHints),
    nullifier_read_request_hints: mapNullifierReadRequestHintsToNoir(inputs.nullifierReadRequestHints),
    key_validation_hints: inputs.keyValidationHints.map(mapKeyValidationHintToNoir) as FixedLengthArray<
      KeyValidationHintNoir,
      KEY_VALIDATION_REQUESTS
    >,
  };
}

export function mapPrivateKernelResetCircuitPrivateInputsToNoir<
  NH_RR_PENDING extends number,
  NH_RR_SETTLED extends number,
  NLL_RR_PENDING extends number,
  NLL_RR_SETTLED extends number,
  KEY_VALIDATION_REQUESTS extends number,
  TAG extends string,
>(
  inputs: PrivateKernelResetCircuitPrivateInputs<
    NH_RR_PENDING,
    NH_RR_SETTLED,
    NLL_RR_PENDING,
    NLL_RR_SETTLED,
    KEY_VALIDATION_REQUESTS,
    TAG
  >,
): PrivateKernelResetCircuitPrivateInputsNoir<
  NH_RR_PENDING,
  NH_RR_SETTLED,
  NLL_RR_PENDING,
  NLL_RR_SETTLED,
  KEY_VALIDATION_REQUESTS
> {
  return {
    previous_kernel: mapPrivateKernelDataToNoir(inputs.previousKernel),
    hints: mapPrivateKernelResetHintsToNoir(inputs.hints),
  };
}

export function mapPrivateKernelTailCircuitPrivateInputsToNoir(
  inputs: PrivateKernelTailCircuitPrivateInputs,
): PrivateKernelTailCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPrivateKernelDataToNoir(inputs.previousKernel),
  };
}

export function mapPrivateKernelTailToPublicCircuitPrivateInputsToNoir(
  inputs: PrivateKernelTailCircuitPrivateInputs,
): PrivateKernelTailToPublicCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPrivateKernelDataToNoir(inputs.previousKernel),
  };
}

export function mapPublicKernelCircuitPrivateInputsToNoir(
  inputs: PublicKernelCircuitPrivateInputs,
): PublicKernelSetupCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPublicKernelDataToNoir(inputs.previousKernel),
    public_call: mapPublicCallDataToNoir(inputs.publicCall),
  };
}

export function mapCombineHintsToNoir(combineHints: CombineHints): CombineHintsNoir {
  return {
    sorted_note_hashes: mapTuple(combineHints.sortedNoteHashes, mapNoteHashToNoir),
    sorted_note_hashes_indexes: mapTuple(combineHints.sortedNoteHashesIndexes, mapNumberToNoir),
    sorted_unencrypted_logs_hashes: mapTuple(combineHints.sortedUnencryptedLogsHashes, mapScopedLogHashToNoir),
    sorted_unencrypted_logs_hashes_indexes: mapTuple(combineHints.sortedUnencryptedLogsHashesIndexes, mapNumberToNoir),
    sorted_public_data_update_requests: mapTuple(
      combineHints.sortedPublicDataUpdateRequests,
      mapPublicDataUpdateRequestToNoir,
    ),
    sorted_public_data_update_requests_indexes: mapTuple(
      combineHints.sortedPublicDataUpdateRequestsIndexes,
      mapNumberToNoir,
    ),
    deduped_public_data_update_requests: mapTuple(
      combineHints.dedupedPublicDataUpdateRequests,
      mapPublicDataUpdateRequestToNoir,
    ),
    deduped_public_data_update_requests_runs: mapTuple(
      combineHints.dedupedPublicDataUpdateRequestsRuns,
      mapNumberToNoir,
    ),
  };
}

export function mapPublicKernelTailCircuitPrivateInputsToNoir(
  inputs: PublicKernelTailCircuitPrivateInputs,
): PublicKernelTailCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPublicKernelDataToNoir(inputs.previousKernel),
    nullifier_read_request_hints: mapNullifierReadRequestHintsToNoir(inputs.nullifierReadRequestHints),
    nullifier_non_existent_read_request_hints: mapNullifierNonExistentReadRequestHintsToNoir(
      inputs.nullifierNonExistentReadRequestHints,
    ),
    public_data_hints: mapTuple(inputs.publicDataHints, mapPublicDataHintToNoir),
    public_data_read_request_hints: mapPublicDataReadRequestHintsToNoir(inputs.publicDataReadRequestHints),
    start_state: mapPartialStateReferenceToNoir(inputs.startState),
    combine_hints: mapCombineHintsToNoir(inputs.combineHints),
  };
}

export function mapPublicKernelCircuitPublicInputsFromNoir(
  inputs: PublicKernelCircuitPublicInputsNoir,
): PublicKernelCircuitPublicInputs {
  return new PublicKernelCircuitPublicInputs(
    mapValidationRequestsFromNoir(inputs.validation_requests),
    mapPublicAccumulatedDataFromNoir(inputs.end_non_revertible),
    mapPublicAccumulatedDataFromNoir(inputs.end),
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapRevertCodeFromNoir(inputs.revert_code),
    mapTupleFromNoir(inputs.public_teardown_call_stack, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, mapCallRequestFromNoir),
    mapAztecAddressFromNoir(inputs.fee_payer),
  );
}

/**
 * Maps a private kernel inputs final to noir.
 * @param storageUpdateRequest - The storage update request.
 * @returns The noir storage update request.
 */
export function mapStorageUpdateRequestToNoir(
  storageUpdateRequest: ContractStorageUpdateRequest,
): StorageUpdateRequestNoir {
  return {
    storage_slot: mapFieldToNoir(storageUpdateRequest.storageSlot),
    new_value: mapFieldToNoir(storageUpdateRequest.newValue),
    counter: mapNumberToNoir(storageUpdateRequest.counter),
  };
}
/**
 * Maps global variables to the noir type.
 * @param globalVariables - The global variables.
 * @returns The noir global variables.
 */
export function mapGlobalVariablesToNoir(globalVariables: GlobalVariables): GlobalVariablesNoir {
  return {
    chain_id: mapFieldToNoir(globalVariables.chainId),
    version: mapFieldToNoir(globalVariables.version),
    block_number: mapFieldToNoir(globalVariables.blockNumber),
    timestamp: mapFieldToNoir(globalVariables.timestamp),
    coinbase: mapEthAddressToNoir(globalVariables.coinbase),
    fee_recipient: mapAztecAddressToNoir(globalVariables.feeRecipient),
    gas_fees: mapGasFeesToNoir(globalVariables.gasFees),
  };
}

/**
 * Maps a storage read to noir.
 * @param storageRead - The storage read.
 * @returns The noir storage read.
 */
export function mapStorageReadToNoir(storageRead: ContractStorageRead): StorageReadNoir {
  return {
    storage_slot: mapFieldToNoir(storageRead.storageSlot),
    current_value: mapFieldToNoir(storageRead.currentValue),
    counter: mapNumberToNoir(storageRead.counter),
  };
}
/**
 * Maps global variables from the noir type.
 * @param globalVariables - The noir global variables.
 * @returns The global variables.
 */
export function mapGlobalVariablesFromNoir(globalVariables: GlobalVariablesNoir): GlobalVariables {
  return new GlobalVariables(
    mapFieldFromNoir(globalVariables.chain_id),
    mapFieldFromNoir(globalVariables.version),
    mapFieldFromNoir(globalVariables.block_number),
    mapFieldFromNoir(globalVariables.timestamp),
    mapEthAddressFromNoir(globalVariables.coinbase),
    mapAztecAddressFromNoir(globalVariables.fee_recipient),
    mapGasFeesFromNoir(globalVariables.gas_fees),
  );
}

export function mapGasFeesToNoir(gasFees: GasFees): GasFeesNoir {
  return {
    fee_per_da_gas: mapFieldToNoir(gasFees.feePerDaGas),
    fee_per_l2_gas: mapFieldToNoir(gasFees.feePerL2Gas),
  };
}

export function mapGasFeesFromNoir(gasFees: GasFeesNoir): GasFees {
  return new GasFees(mapFieldFromNoir(gasFees.fee_per_da_gas), mapFieldFromNoir(gasFees.fee_per_l2_gas));
}

/**
 * Maps a constant rollup data to a noir constant rollup data.
 * @param constantRollupData - The circuits.js constant rollup data.
 * @returns The noir constant rollup data.
 */
export function mapConstantRollupDataToNoir(constantRollupData: ConstantRollupData): ConstantRollupDataNoir {
  return {
    last_archive: mapAppendOnlyTreeSnapshotToNoir(constantRollupData.lastArchive),
    vk_tree_root: mapFieldToNoir(constantRollupData.vkTreeRoot),
    global_variables: mapGlobalVariablesToNoir(constantRollupData.globalVariables),
  };
}

/**
 * Maps a public circuit public inputs to noir.
 * @param publicInputs - The public circuit public inputs.
 * @returns The noir public circuit public inputs.
 */
export function mapPublicCircuitPublicInputsToNoir(
  publicInputs: PublicCircuitPublicInputs,
): PublicCircuitPublicInputsNoir {
  return {
    call_context: mapCallContextToNoir(publicInputs.callContext),
    args_hash: mapFieldToNoir(publicInputs.argsHash),
    returns_hash: mapFieldToNoir(publicInputs.returnsHash),
    note_hash_read_requests: mapTuple(publicInputs.noteHashReadRequests, mapReadRequestToNoir),
    nullifier_read_requests: mapTuple(publicInputs.nullifierReadRequests, mapReadRequestToNoir),
    nullifier_non_existent_read_requests: mapTuple(publicInputs.nullifierNonExistentReadRequests, mapReadRequestToNoir),
    l1_to_l2_msg_read_requests: mapTuple(publicInputs.l1ToL2MsgReadRequests, mapReadRequestToNoir),
    contract_storage_update_requests: mapTuple(
      publicInputs.contractStorageUpdateRequests,
      mapStorageUpdateRequestToNoir,
    ),
    contract_storage_reads: mapTuple(publicInputs.contractStorageReads, mapStorageReadToNoir),
    public_call_stack_hashes: mapTuple(publicInputs.publicCallStackHashes, mapFieldToNoir),
    note_hashes: mapTuple(publicInputs.noteHashes, mapNoteHashToNoir),
    nullifiers: mapTuple(publicInputs.nullifiers, mapNullifierToNoir),
    l2_to_l1_msgs: mapTuple(publicInputs.l2ToL1Msgs, mapL2ToL1MessageToNoir),
    start_side_effect_counter: mapFieldToNoir(publicInputs.startSideEffectCounter),
    end_side_effect_counter: mapFieldToNoir(publicInputs.endSideEffectCounter),
    unencrypted_logs_hashes: mapTuple(publicInputs.unencryptedLogsHashes, mapLogHashToNoir),
    historical_header: mapHeaderToNoir(publicInputs.historicalHeader),
    global_variables: mapGlobalVariablesToNoir(publicInputs.globalVariables),
    prover_address: mapAztecAddressToNoir(publicInputs.proverAddress),
    revert_code: mapRevertCodeToNoir(publicInputs.revertCode),
    start_gas_left: mapGasToNoir(publicInputs.startGasLeft),
    end_gas_left: mapGasToNoir(publicInputs.endGasLeft),
    transaction_fee: mapFieldToNoir(publicInputs.transactionFee),
  };
}
/**
 * Maps a constant rollup data from noir to the circuits.js type.
 * @param constantRollupData - The noir constant rollup data.
 * @returns The circuits.js constant rollup data.
 */
export function mapConstantRollupDataFromNoir(constantRollupData: ConstantRollupDataNoir): ConstantRollupData {
  return new ConstantRollupData(
    mapAppendOnlyTreeSnapshotFromNoir(constantRollupData.last_archive),
    mapFieldFromNoir(constantRollupData.vk_tree_root),
    mapGlobalVariablesFromNoir(constantRollupData.global_variables),
  );
}

/**
 * Maps a base or merge rollup public inputs to a noir base or merge rollup public inputs.
 * @param baseOrMergeRollupPublicInputs - The base or merge rollup public inputs.
 * @returns The noir base or merge rollup public inputs.
 */
export function mapBaseOrMergeRollupPublicInputsToNoir(
  baseOrMergeRollupPublicInputs: BaseOrMergeRollupPublicInputs,
): BaseOrMergeRollupPublicInputsNoir {
  return {
    rollup_type: mapFieldToNoir(new Fr(baseOrMergeRollupPublicInputs.rollupType)),
    num_txs: mapFieldToNoir(new Fr(baseOrMergeRollupPublicInputs.numTxs)),
    constants: mapConstantRollupDataToNoir(baseOrMergeRollupPublicInputs.constants),
    start: mapPartialStateReferenceToNoir(baseOrMergeRollupPublicInputs.start),
    end: mapPartialStateReferenceToNoir(baseOrMergeRollupPublicInputs.end),
    txs_effects_hash: mapFieldToNoir(baseOrMergeRollupPublicInputs.txsEffectsHash),
    out_hash: mapFieldToNoir(baseOrMergeRollupPublicInputs.outHash),
    accumulated_fees: mapFieldToNoir(baseOrMergeRollupPublicInputs.accumulatedFees),
  };
}

/**
 * Maps a public call stack item to noir.
 * @param publicCallStackItem - The public call stack item.
 * @returns The noir public call stack item.
 */
export function mapPublicCallStackItemToNoir(publicCallStackItem: PublicCallStackItem): PublicCallStackItemNoir {
  return {
    contract_address: mapAztecAddressToNoir(publicCallStackItem.contractAddress),
    public_inputs: mapPublicCircuitPublicInputsToNoir(publicCallStackItem.publicInputs),
    is_execution_request: publicCallStackItem.isExecutionRequest,
    function_data: mapFunctionDataToNoir(publicCallStackItem.functionData),
  };
}

/**
 * Maps a public call data to noir.
 * @param publicCall - The public call data.
 * @returns The noir public call data.
 */
export function mapPublicCallDataToNoir(publicCall: PublicCallData): PublicCallDataNoir {
  return {
    call_stack_item: mapPublicCallStackItemToNoir(publicCall.callStackItem),
    public_call_stack: mapTuple(publicCall.publicCallStack, mapCallRequestToNoir),
    proof: {},
    bytecode_hash: mapFieldToNoir(publicCall.bytecodeHash),
  };
}
/**
 * Maps a base or merge rollup public inputs from noir to the circuits.js type.
 * @param baseOrMergeRollupPublicInputs - The noir base or merge rollup public inputs.
 * @returns The circuits.js base or merge rollup public inputs.
 */
export function mapBaseOrMergeRollupPublicInputsFromNoir(
  baseOrMergeRollupPublicInputs: BaseOrMergeRollupPublicInputsNoir,
): BaseOrMergeRollupPublicInputs {
  return new BaseOrMergeRollupPublicInputs(
    mapNumberFromNoir(baseOrMergeRollupPublicInputs.rollup_type),
    mapNumberFromNoir(baseOrMergeRollupPublicInputs.num_txs),
    mapConstantRollupDataFromNoir(baseOrMergeRollupPublicInputs.constants),
    mapPartialStateReferenceFromNoir(baseOrMergeRollupPublicInputs.start),
    mapPartialStateReferenceFromNoir(baseOrMergeRollupPublicInputs.end),
    mapFieldFromNoir(baseOrMergeRollupPublicInputs.txs_effects_hash),
    mapFieldFromNoir(baseOrMergeRollupPublicInputs.out_hash),
    mapFieldFromNoir(baseOrMergeRollupPublicInputs.accumulated_fees),
  );
}

/**
 * Maps a previous rollup data from the circuits.js type to noir.
 * @param previousRollupData - The circuits.js previous rollup data.
 * @returns The noir previous rollup data.
 */
export function mapPreviousRollupDataToNoir(previousRollupData: PreviousRollupData): PreviousRollupDataNoir {
  return {
    base_or_merge_rollup_public_inputs: mapBaseOrMergeRollupPublicInputsToNoir(
      previousRollupData.baseOrMergeRollupPublicInputs,
    ),
    proof: mapRecursiveProofToNoir(previousRollupData.proof),
    vk: mapVerificationKeyToNoir(previousRollupData.vk),
    vk_witness: {
      leaf_index: mapFieldToNoir(new Fr(previousRollupData.vkWitness.leafIndex)),
      sibling_path: mapTuple(previousRollupData.vkWitness.siblingPath, mapFieldToNoir),
    },
  };
}

/**
 * Maps a AOT snapshot to noir.
 * @param snapshot - The circuits.js AOT snapshot.
 * @returns The noir AOT snapshot.
 */
export function mapAppendOnlyTreeSnapshotFromNoir(snapshot: AppendOnlyTreeSnapshotNoir): AppendOnlyTreeSnapshot {
  return new AppendOnlyTreeSnapshot(
    mapFieldFromNoir(snapshot.root),
    mapNumberFromNoir(snapshot.next_available_leaf_index),
  );
}

/**
 * Maps a AOT snapshot from noir to the circuits.js type.
 * @param snapshot - The noir AOT snapshot.
 * @returns The circuits.js AOT snapshot.
 */
export function mapAppendOnlyTreeSnapshotToNoir(snapshot: AppendOnlyTreeSnapshot): AppendOnlyTreeSnapshotNoir {
  return {
    root: mapFieldToNoir(snapshot.root),
    next_available_leaf_index: mapFieldToNoir(new Fr(snapshot.nextAvailableLeafIndex)),
  };
}

export function mapRootRollupParityInputToNoir(
  rootParityInput: RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
): RootRollupParityInputNoir {
  return {
    proof: mapRecursiveProofToNoir(rootParityInput.proof),
    verification_key: mapVerificationKeyToNoir(rootParityInput.verificationKey),
    vk_path: mapTuple(rootParityInput.vkPath, mapFieldToNoir),
    public_inputs: mapParityPublicInputsToNoir(rootParityInput.publicInputs),
  };
}

/**
 * Naos the root rollup inputs to noir.
 * @param rootRollupInputs - The circuits.js root rollup inputs.
 * @returns The noir root rollup inputs.
 */
export function mapRootRollupInputsToNoir(rootRollupInputs: RootRollupInputs): RootRollupInputsNoir {
  return {
    previous_rollup_data: mapTuple(rootRollupInputs.previousRollupData, mapPreviousRollupDataToNoir),
    l1_to_l2_roots: mapRootRollupParityInputToNoir(rootRollupInputs.l1ToL2Roots),
    l1_to_l2_messages: mapTuple(rootRollupInputs.newL1ToL2Messages, mapFieldToNoir),
    l1_to_l2_message_subtree_sibling_path: mapTuple(
      rootRollupInputs.newL1ToL2MessageTreeRootSiblingPath,
      mapFieldToNoir,
    ),
    start_l1_to_l2_message_tree_snapshot: mapAppendOnlyTreeSnapshotToNoir(
      rootRollupInputs.startL1ToL2MessageTreeSnapshot,
    ),
    start_archive_snapshot: mapAppendOnlyTreeSnapshotToNoir(rootRollupInputs.startArchiveSnapshot),
    new_archive_sibling_path: mapTuple(rootRollupInputs.newArchiveSiblingPath, mapFieldToNoir),
  };
}

export function mapRecursiveProofToNoir<PROOF_LENGTH extends number>(proof: RecursiveProof<PROOF_LENGTH>) {
  return {
    fields: mapTuple(proof.proof, mapFieldToNoir),
  };
}

export function mapRootParityInputToNoir(
  rootParityInput: RootParityInput<typeof RECURSIVE_PROOF_LENGTH>,
): ParityRootParityInputNoir {
  return {
    proof: mapRecursiveProofToNoir(rootParityInput.proof),
    verification_key: mapVerificationKeyToNoir(rootParityInput.verificationKey),
    vk_path: mapTuple(rootParityInput.vkPath, mapFieldToNoir),
    public_inputs: mapParityPublicInputsToNoir(rootParityInput.publicInputs),
  };
}

export function mapParityPublicInputsToNoir(parityPublicInputs: ParityPublicInputs): ParityPublicInputsNoir {
  return {
    sha_root: mapFieldToNoir(parityPublicInputs.shaRoot),
    converted_root: mapFieldToNoir(parityPublicInputs.convertedRoot),
    vk_tree_root: mapFieldToNoir(parityPublicInputs.vkTreeRoot),
  };
}

/**
 * Maps a root rollup public inputs from noir.
 * @param rootRollupPublicInputs - The noir root rollup public inputs.
 * @returns The circuits.js root rollup public inputs.
 */
export function mapRootRollupPublicInputsFromNoir(
  rootRollupPublicInputs: RootRollupPublicInputsNoir,
): RootRollupPublicInputs {
  return new RootRollupPublicInputs(
    mapAppendOnlyTreeSnapshotFromNoir(rootRollupPublicInputs.archive),
    mapFieldFromNoir(rootRollupPublicInputs.vk_tree_root),
    mapHeaderFromNoir(rootRollupPublicInputs.header),
  );
}

/**
 * Maps a parity public inputs from noir.
 * @param parityPublicInputs - The noir parity public inputs.
 * @returns The circuits.js parity public inputs.
 */
export function mapParityPublicInputsFromNoir(parityPublicInputs: ParityPublicInputsNoir): ParityPublicInputs {
  return new ParityPublicInputs(
    mapFieldFromNoir(parityPublicInputs.sha_root),
    mapFieldFromNoir(parityPublicInputs.converted_root),
    mapFieldFromNoir(parityPublicInputs.vk_tree_root),
  );
}

/**
 * Maps header to Noir
 * @param header - The header.
 * @returns Header.
 */
export function mapHeaderToNoir(header: Header): HeaderNoir {
  return {
    last_archive: mapAppendOnlyTreeSnapshotToNoir(header.lastArchive),
    content_commitment: mapContentCommitmentToNoir(header.contentCommitment),
    state: mapStateReferenceToNoir(header.state),
    global_variables: mapGlobalVariablesToNoir(header.globalVariables),
    total_fees: mapFieldToNoir(header.totalFees),
  };
}

/**
 * Maps header from Noir.
 * @param header - The header.
 * @returns Header.
 */
export function mapHeaderFromNoir(header: HeaderNoir): Header {
  return new Header(
    mapAppendOnlyTreeSnapshotFromNoir(header.last_archive),
    mapContentCommitmentFromNoir(header.content_commitment),
    mapStateReferenceFromNoir(header.state),
    mapGlobalVariablesFromNoir(header.global_variables),
    mapFieldFromNoir(header.total_fees),
  );
}

/**
 * Maps a content commitment to Noir
 *
 */
export function mapContentCommitmentToNoir(contentCommitment: ContentCommitment): ContentCommitmentNoir {
  return {
    num_txs: mapFieldToNoir(contentCommitment.numTxs),
    txs_effects_hash: mapSha256HashToNoir(contentCommitment.txsEffectsHash),
    in_hash: mapSha256HashToNoir(contentCommitment.inHash),
    out_hash: mapSha256HashToNoir(contentCommitment.outHash),
  };
}

/**
 * Maps a content commitment to Noir
 *
 */
export function mapContentCommitmentFromNoir(contentCommitment: ContentCommitmentNoir): ContentCommitment {
  return new ContentCommitment(
    mapFieldFromNoir(contentCommitment.num_txs),
    mapSha256HashFromNoir(contentCommitment.txs_effects_hash),
    mapSha256HashFromNoir(contentCommitment.in_hash),
    mapSha256HashFromNoir(contentCommitment.out_hash),
  );
}

/**
 * Maps state reference to Noir.
 * @param stateReference - The state reference.
 * @returns Noir representation of state reference.
 */
export function mapStateReferenceToNoir(stateReference: StateReference): StateReferenceNoir {
  return {
    l1_to_l2_message_tree: mapAppendOnlyTreeSnapshotToNoir(stateReference.l1ToL2MessageTree),
    partial: mapPartialStateReferenceToNoir(stateReference.partial),
  };
}

/**
 * Maps state reference from Noir.
 * @param stateReference - The state reference.
 * @returns State reference
 */
export function mapStateReferenceFromNoir(stateReference: StateReferenceNoir): StateReference {
  return new StateReference(
    mapAppendOnlyTreeSnapshotFromNoir(stateReference.l1_to_l2_message_tree),
    mapPartialStateReferenceFromNoir(stateReference.partial),
  );
}

/**
 * Maps partial state reference from Noir.
 * @param partialStateReference - The state reference.
 * @returns Partial state reference
 */
export function mapPartialStateReferenceFromNoir(
  partialStateReference: PartialStateReferenceNoir,
): PartialStateReference {
  return new PartialStateReference(
    mapAppendOnlyTreeSnapshotFromNoir(partialStateReference.note_hash_tree),
    mapAppendOnlyTreeSnapshotFromNoir(partialStateReference.nullifier_tree),
    mapAppendOnlyTreeSnapshotFromNoir(partialStateReference.public_data_tree),
  );
}

/**
 * Maps the merge rollup inputs to noir.
 * @param mergeRollupInputs - The circuits.js merge rollup inputs.
 * @returns The noir merge rollup inputs.
 */
export function mapMergeRollupInputsToNoir(mergeRollupInputs: MergeRollupInputs): MergeRollupInputsNoir {
  return {
    previous_rollup_data: mapTuple(mergeRollupInputs.previousRollupData, mapPreviousRollupDataToNoir),
  };
}

function mapNoteHashLeafPreimageToNoir(noteHashLeafValue: Fr): NoteHashLeafPreimageNoir {
  return {
    value: mapFieldToNoir(noteHashLeafValue),
  };
}

/**
 * Maps a nullifier leaf preimage to noir
 * @param nullifierLeafPreimage - The nullifier leaf preimage.
 * @returns The noir nullifier leaf preimage.
 */
export function mapNullifierLeafPreimageToNoir(
  nullifierLeafPreimage: NullifierLeafPreimage,
): NullifierLeafPreimageNoir {
  return {
    nullifier: mapFieldToNoir(nullifierLeafPreimage.nullifier),
    next_nullifier: mapFieldToNoir(nullifierLeafPreimage.nextNullifier),
    next_index: mapNumberToNoir(Number(nullifierLeafPreimage.nextIndex)),
  };
}

function mapMembershipWitnessToNoir<N extends number>(witness: MembershipWitness<N>): MembershipWitnessNoir<N> {
  const siblingPath = mapTuple(witness.siblingPath, mapFieldToNoir) as FixedLengthArray<NoirField, N>;
  return {
    leaf_index: witness.leafIndex.toString(),
    sibling_path: siblingPath,
  };
}

/**
 * Maps a leaf of the public data tree to noir.
 */
export function mapPublicDataTreeLeafToNoir(leaf: PublicDataTreeLeaf): PublicDataTreeLeafNoir {
  return {
    slot: mapFieldToNoir(leaf.slot),
    value: mapFieldToNoir(leaf.value),
  };
}

/**
 * Maps a leaf preimage of the public data tree to noir.
 */
export function mapPublicDataTreePreimageToNoir(preimage: PublicDataTreeLeafPreimage): PublicDataTreeLeafPreimageNoir {
  return {
    slot: mapFieldToNoir(preimage.slot),
    value: mapFieldToNoir(preimage.value),
    next_slot: mapFieldToNoir(preimage.nextSlot),
    next_index: mapNumberToNoir(Number(preimage.nextIndex)),
  };
}

/**
 * Maps a partial state reference to a noir partial state reference.
 * @param partialStateReference - The partial state reference.
 * @returns The noir partial state reference.
 */
export function mapPartialStateReferenceToNoir(
  partialStateReference: PartialStateReference,
): PartialStateReferenceNoir {
  return {
    note_hash_tree: mapAppendOnlyTreeSnapshotToNoir(partialStateReference.noteHashTree),
    nullifier_tree: mapAppendOnlyTreeSnapshotToNoir(partialStateReference.nullifierTree),
    public_data_tree: mapAppendOnlyTreeSnapshotToNoir(partialStateReference.publicDataTree),
  };
}

/**
 * Maps state diff hints to a noir state diff hints.
 * @param hints - The state diff hints.
 * @returns The noir state diff hints.
 */
export function mapStateDiffHintsToNoir(hints: StateDiffHints): StateDiffHintsNoir {
  return {
    nullifier_predecessor_preimages: mapTuple(hints.nullifierPredecessorPreimages, mapNullifierLeafPreimageToNoir),
    nullifier_predecessor_membership_witnesses: mapTuple(
      hints.nullifierPredecessorMembershipWitnesses,
      (witness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>) => mapMembershipWitnessToNoir(witness),
    ),
    sorted_nullifiers: mapTuple(hints.sortedNullifiers, mapFieldToNoir),
    sorted_nullifier_indexes: mapTuple(hints.sortedNullifierIndexes, (index: number) => mapNumberToNoir(index)),
    note_hash_subtree_sibling_path: mapTuple(hints.noteHashSubtreeSiblingPath, mapFieldToNoir),
    nullifier_subtree_sibling_path: mapTuple(hints.nullifierSubtreeSiblingPath, mapFieldToNoir),
    public_data_sibling_path: mapTuple(hints.publicDataSiblingPath, mapFieldToNoir),
  };
}

/**
 * Maps base parity inputs to noir.
 * @param inputs - The circuits.js base parity inputs.
 * @returns The noir base parity inputs.
 */
export function mapBaseParityInputsToNoir(inputs: BaseParityInputs): BaseParityInputsNoir {
  return {
    msgs: mapTuple(inputs.msgs, mapFieldToNoir),
    vk_tree_root: mapFieldToNoir(inputs.vkTreeRoot),
  };
}

/**
 * Maps root parity inputs to noir.
 * @param inputs - The circuits.js root parity inputs.
 * @returns The noir root parity inputs.
 */
export function mapRootParityInputsToNoir(inputs: RootParityInputs): RootParityInputsNoir {
  return {
    children: mapTuple(inputs.children, mapRootParityInputToNoir),
  };
}

/**
 * Maps the inputs to the base rollup to noir.
 * @param input - The circuits.js base rollup inputs.
 * @returns The noir base rollup inputs.
 */
export function mapBaseRollupInputsToNoir(inputs: BaseRollupInputs): BaseRollupInputsNoir {
  return {
    kernel_data: mapKernelDataToNoir(inputs.kernelData),
    start: mapPartialStateReferenceToNoir(inputs.start),
    state_diff_hints: mapStateDiffHintsToNoir(inputs.stateDiffHints),

    sorted_public_data_writes: mapTuple(inputs.sortedPublicDataWrites, mapPublicDataTreeLeafToNoir),

    sorted_public_data_writes_indexes: mapTuple(inputs.sortedPublicDataWritesIndexes, mapNumberToNoir),

    low_public_data_writes_preimages: mapTuple(inputs.lowPublicDataWritesPreimages, mapPublicDataTreePreimageToNoir),

    low_public_data_writes_witnesses: mapTuple(
      inputs.lowPublicDataWritesMembershipWitnesses,
      (witness: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>) => mapMembershipWitnessToNoir(witness),
    ),

    archive_root_membership_witness: mapMembershipWitnessToNoir(inputs.archiveRootMembershipWitness),
    constants: mapConstantRollupDataToNoir(inputs.constants),
    fee_payer_gas_token_balance_read_hint: mapPublicDataHintToNoir(inputs.feePayerGasTokenBalanceReadHint),
  };
}

export function mapEmptyKernelInputsToNoir(inputs: PrivateKernelEmptyInputs): PrivateKernelEmptyPrivateInputsNoir {
  return {
    empty_nested: mapEmptyNestedDataToNoir(inputs.emptyNested),
    historical_header: mapHeaderToNoir(inputs.header),
    chain_id: mapFieldToNoir(inputs.chainId),
    version: mapFieldToNoir(inputs.version),
    vk_tree_root: mapFieldToNoir(inputs.vkTreeRoot),
  };
}

function mapEmptyNestedDataToNoir(inputs: EmptyNestedData): EmptyNestedDataNoir {
  return {
    proof: mapRecursiveProofToNoir(inputs.proof),
    vk: mapVerificationKeyToNoir(inputs.vk),
  };
}

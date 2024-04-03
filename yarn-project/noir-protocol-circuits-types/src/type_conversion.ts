import {
  type ARCHIVE_HEIGHT,
  AggregationObject,
  AppendOnlyTreeSnapshot,
  AztecAddress,
  BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  CallContext,
  CallRequest,
  CallerContext,
  CombinedAccumulatedData,
  CombinedConstantData,
  ConstantRollupData,
  ContentCommitment,
  type ContractStorageRead,
  type ContractStorageUpdateRequest,
  EthAddress,
  type FUNCTION_TREE_HEIGHT,
  Fr,
  FunctionData,
  FunctionSelector,
  GlobalVariables,
  type GrumpkinPrivateKey,
  GrumpkinScalar,
  Header,
  KernelCircuitPublicInputs,
  type KernelData,
  type L2ToL1Message,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MaxBlockNumber,
  type MembershipWitness,
  type MergeRollupInputs,
  type NULLIFIER_TREE_HEIGHT,
  NUM_BYTES_PER_SHA256,
  type NonMembershipHint,
  type NoteHashReadRequestMembershipWitness,
  NullifierKeyValidationRequest,
  NullifierKeyValidationRequestContext,
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
  type PrivateCallStackItem,
  type PrivateCircuitPublicInputs,
  PrivateKernelCircuitPublicInputs,
  type PrivateKernelData,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelTailCircuitPrivateInputs,
  PrivateKernelTailCircuitPublicInputs,
  PublicAccumulatedData,
  type PublicCallData,
  type PublicCallStackItem,
  type PublicCircuitPublicInputs,
  PublicDataRead,
  type PublicDataTreeLeaf,
  type PublicDataTreeLeafPreimage,
  PublicDataUpdateRequest,
  type PublicKernelCircuitPrivateInputs,
  PublicKernelCircuitPublicInputs,
  type PublicKernelData,
  type PublicKernelTailCircuitPrivateInputs,
  ReadRequest,
  ReadRequestContext,
  type ReadRequestStatus,
  RevertCode,
  RollupValidationRequests,
  type RootParityInput,
  type RootParityInputs,
  type RootRollupInputs,
  RootRollupPublicInputs,
  type SettledReadHint,
  SideEffect,
  SideEffectLinkedToNoteHash,
  type StateDiffHints,
  StateReference,
  TxContext,
  type TxRequest,
  ValidationRequests,
} from '@aztec/circuits.js';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { type Tuple, mapTuple, toTruncField } from '@aztec/foundation/serialize';

import { type BaseParityInputs as BaseParityInputsNoir } from './types/parity_base_types.js';
import { type RootParityInputs as RootParityInputsNoir } from './types/parity_root_types.js';
import {
  type CallContext as CallContextNoir,
  type CallRequest as CallRequestNoir,
  type CallerContext as CallerContextNoir,
  type CombinedConstantData as CombinedConstantDataNoir,
  type FunctionData as FunctionDataNoir,
  type FunctionLeafMembershipWitness as FunctionLeafMembershipWitnessNoir,
  type FunctionSelector as FunctionSelectorNoir,
  type GrumpkinPrivateKey as GrumpkinPrivateKeyNoir,
  type L2ToL1Message as L2ToL1MessageNoir,
  type MaxBlockNumber as MaxBlockNumberNoir,
  type AztecAddress as NoirAztecAddress,
  type EthAddress as NoirEthAddress,
  type Field as NoirField,
  type GrumpkinPoint as NoirPoint,
  type NoteHashReadRequestMembershipWitness as NoteHashReadRequestMembershipWitnessNoir,
  type NullifierKeyValidationRequestContext as NullifierKeyValidationRequestContextNoir,
  type NullifierKeyValidationRequest as NullifierKeyValidationRequestNoir,
  type PrivateAccumulatedData as PrivateAccumulatedDataNoir,
  type PrivateCallData as PrivateCallDataNoir,
  type PrivateCallStackItem as PrivateCallStackItemNoir,
  type PrivateCircuitPublicInputs as PrivateCircuitPublicInputsNoir,
  type PrivateKernelCircuitPublicInputs as PrivateKernelCircuitPublicInputsNoir,
  type PrivateKernelInitCircuitPrivateInputs as PrivateKernelInitCircuitPrivateInputsNoir,
  type PublicDataRead as PublicDataReadNoir,
  type ReadRequestContext as ReadRequestContextNoir,
  type ReadRequest as ReadRequestNoir,
  type RollupValidationRequests as RollupValidationRequestsNoir,
  type SideEffectLinkedToNoteHash as SideEffectLinkedToNoteHashNoir,
  type SideEffect as SideEffectNoir,
  type TxContext as TxContextNoir,
  type TxRequest as TxRequestNoir,
  type ValidationRequests as ValidationRequestsNoir,
} from './types/private_kernel_init_types.js';
import { type PrivateKernelInnerCircuitPrivateInputs as PrivateKernelInnerCircuitPrivateInputsNoir } from './types/private_kernel_inner_types.js';
import { type PrivateKernelTailToPublicCircuitPrivateInputs as PrivateKernelTailToPublicCircuitPrivateInputsNoir } from './types/private_kernel_tail_to_public_types.js';
import {
  type CombinedAccumulatedData as CombinedAccumulatedDataNoir,
  type NullifierReadRequestHints as NullifierReadRequestHintsNoir,
  type NullifierSettledReadHint as NullifierSettledReadHintNoir,
  type PendingReadHint as PendingReadHintNoir,
  type PrivateKernelData as PrivateKernelDataNoir,
  type PrivateKernelTailCircuitPrivateInputs as PrivateKernelTailCircuitPrivateInputsNoir,
  type ReadRequestStatus as ReadRequestStatusNoir,
} from './types/private_kernel_tail_types.js';
import {
  type PublicAccumulatedData as PublicAccumulatedDataNoir,
  type PublicKernelData as PublicKernelDataNoir,
} from './types/public_kernel_app_logic_types.js';
import {
  type PublicCallData as PublicCallDataNoir,
  type PublicCallStackItem as PublicCallStackItemNoir,
  type PublicCircuitPublicInputs as PublicCircuitPublicInputsNoir,
  type PublicKernelCircuitPublicInputs as PublicKernelCircuitPublicInputsNoir,
  type PublicKernelSetupCircuitPrivateInputs as PublicKernelSetupCircuitPrivateInputsNoir,
  type StorageRead as StorageReadNoir,
  type StorageUpdateRequest as StorageUpdateRequestNoir,
} from './types/public_kernel_setup_types.js';
import {
  type NullifierNonExistentReadRequestHints as NullifierNonExistentReadRequestHintsNoir,
  type NullifierNonMembershipHint as NullifierNonMembershipHintNoir,
  type PublicDataUpdateRequest as PublicDataUpdateRequestNoir,
  type PublicKernelTailCircuitPrivateInputs as PublicKernelTailCircuitPrivateInputsNoir,
} from './types/public_kernel_tail_types.js';
import {
  type ArchiveRootMembershipWitness as ArchiveRootMembershipWitnessNoir,
  type BaseRollupInputs as BaseRollupInputsNoir,
  type KernelCircuitPublicInputs as KernelCircuitPublicInputsNoir,
  type KernelData as KernelDataNoir,
  type NullifierLeafPreimage as NullifierLeafPreimageNoir,
  type NullifierMembershipWitness as NullifierMembershipWitnessNoir,
  type PublicDataMembershipWitness as PublicDataMembershipWitnessNoir,
  type PublicDataTreeLeaf as PublicDataTreeLeafNoir,
  type PublicDataTreeLeafPreimage as PublicDataTreeLeafPreimageNoir,
  type StateDiffHints as StateDiffHintsNoir,
} from './types/rollup_base_types.js';
import { type MergeRollupInputs as MergeRollupInputsNoir } from './types/rollup_merge_types.js';
import {
  type AppendOnlyTreeSnapshot as AppendOnlyTreeSnapshotNoir,
  type BaseOrMergeRollupPublicInputs as BaseOrMergeRollupPublicInputsNoir,
  type ConstantRollupData as ConstantRollupDataNoir,
  type ContentCommitment as ContentCommitmentNoir,
  type Field,
  type GlobalVariables as GlobalVariablesNoir,
  type Header as HeaderNoir,
  type ParityPublicInputs as ParityPublicInputsNoir,
  type PartialStateReference as PartialStateReferenceNoir,
  type PreviousRollupData as PreviousRollupDataNoir,
  type RootParityInput as RootParityInputNoir,
  type RootRollupInputs as RootRollupInputsNoir,
  type RootRollupPublicInputs as RootRollupPublicInputsNoir,
  type StateReference as StateReferenceNoir,
} from './types/rollup_root_types.js';

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
  };
}

/**
 * Maps a noir point to a point.
 * @param point - The noir point.
 * @returns The point.
 */
export function mapPointFromNoir(point: NoirPoint): Point {
  return new Point(mapFieldFromNoir(point.x), mapFieldFromNoir(point.y));
}

/**
 * Maps a GrumpkinPrivateKey to a noir GrumpkinPrivateKey.
 * @param privateKey - The GrumpkinPrivateKey.
 * @returns The noir GrumpkinPrivateKey.
 */
export function mapGrumpkinPrivateKeyToNoir(privateKey: GrumpkinPrivateKey): GrumpkinPrivateKeyNoir {
  return {
    high: mapFieldToNoir(privateKey.high),
    low: mapFieldToNoir(privateKey.low),
  };
}

/**
 * Maps a noir GrumpkinPrivateKey to a GrumpkinPrivateKey.
 * @param privateKey - The noir GrumpkinPrivateKey.
 * @returns The GrumpkinPrivateKey.
 */
export function mapGrumpkinPrivateKeyFromNoir(privateKey: GrumpkinPrivateKeyNoir): GrumpkinPrivateKey {
  return GrumpkinScalar.fromHighLow(mapFieldFromNoir(privateKey.high), mapFieldFromNoir(privateKey.low));
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
    is_fee_payment_tx: txContext.isFeePaymentTx,
    is_rebate_payment_tx: txContext.isRebatePaymentTx,
    chain_id: mapFieldToNoir(txContext.chainId),
    version: mapFieldToNoir(txContext.version),
  };
}

/**
 * Maps a noir tx context to a tx context.
 * @param txContext - The noir tx context.
 * @returns The tx context.
 */
export function mapTxContextFromNoir(txContext: TxContextNoir): TxContext {
  return new TxContext(
    txContext.is_fee_payment_tx,
    txContext.is_rebate_payment_tx,
    mapFieldFromNoir(txContext.chain_id),
    mapFieldFromNoir(txContext.version),
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
    mapEthAddressFromNoir(callContext.portal_contract_address),
    mapFunctionSelectorFromNoir(callContext.function_selector),
    callContext.is_delegate_call,
    callContext.is_static_call,
    mapNumberFromNoir(callContext.side_effect_counter),
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
    portal_contract_address: mapEthAddressToNoir(callContext.portalContractAddress),
    function_selector: mapFunctionSelectorToNoir(callContext.functionSelector),
    is_delegate_call: callContext.isDelegateCall,
    is_static_call: callContext.isStaticCall,
    side_effect_counter: mapNumberToNoir(callContext.sideEffectCounter),
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

/**
 * Maps a SideEffect to a noir side effect.
 * @param sideEffect - The SideEffect.
 * @returns The noir side effect.
 */
export function mapSideEffectToNoir(sideEffect: SideEffect): SideEffectNoir {
  return {
    value: mapFieldToNoir(sideEffect.value),
    counter: mapFieldToNoir(sideEffect.counter),
  };
}

/**
 * Maps a noir side effect to a SideEffect.
 * @param sideEffect - The noir SideEffect.
 * @returns The TS side effect.
 */
export function mapSideEffectFromNoir(sideEffect: SideEffectNoir): SideEffect {
  return new SideEffect(mapFieldFromNoir(sideEffect.value), mapFieldFromNoir(sideEffect.counter));
}

/**
 * Maps a SideEffectLinked to a noir side effect.
 * @param sideEffectLinked - The side effect linked to note hash.
 * @returns The noir SideEffectLinkedToNoteHash.
 */
export function mapSideEffectLinkedToNoir(
  sideEffectLinked: SideEffectLinkedToNoteHash,
): SideEffectLinkedToNoteHashNoir {
  return {
    value: mapFieldToNoir(sideEffectLinked.value),
    note_hash: mapFieldToNoir(sideEffectLinked.noteHash),
    counter: mapFieldToNoir(sideEffectLinked.counter),
  };
}

/**
 * Maps a noir side effect to aSideEffect.
 * @param sideEffect - The noir side effect.
 * @returns The TS side effect.
 */
export function mapSideEffectLinkedFromNoir(
  sideEffectLinked: SideEffectLinkedToNoteHashNoir,
): SideEffectLinkedToNoteHash {
  return new SideEffectLinkedToNoteHash(
    mapFieldFromNoir(sideEffectLinked.value),
    mapFieldFromNoir(sideEffectLinked.note_hash),
    mapFieldFromNoir(sideEffectLinked.counter),
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

/**
 * Maps a ReadRequestContext to a noir ReadRequestContext.
 * @param readRequestContext - The read request context.
 * @returns The noir ReadRequestContext.
 */
export function mapReadRequestContextToNoir(readRequestContext: ReadRequestContext): ReadRequestContextNoir {
  return {
    value: mapFieldToNoir(readRequestContext.value),
    counter: mapNumberToNoir(readRequestContext.counter),
    contract_address: mapAztecAddressToNoir(readRequestContext.contractAddress),
  };
}

/**
 * Maps a noir ReadRequest to ReadRequest.
 * @param readRequest - The noir ReadRequest.
 * @returns The TS ReadRequest.
 */
export function mapReadRequestContextFromNoir(readRequestContext: ReadRequestContextNoir): ReadRequestContext {
  return new ReadRequestContext(
    mapFieldFromNoir(readRequestContext.value),
    mapNumberFromNoir(readRequestContext.counter),
    mapAztecAddressFromNoir(readRequestContext.contract_address),
  );
}

/**
 * Maps a NullifierKeyValidationRequest to a noir NullifierKeyValidationRequest.
 * @param request - The NullifierKeyValidationRequest.
 * @returns The noir NullifierKeyValidationRequest.
 */
export function mapNullifierKeyValidationRequestToNoir(
  request: NullifierKeyValidationRequest,
): NullifierKeyValidationRequestNoir {
  return {
    public_key: mapPointToNoir(request.publicKey),
    secret_key: mapGrumpkinPrivateKeyToNoir(request.secretKey),
  };
}

/**
 * Maps a noir NullifierKeyValidationRequest to NullifierKeyValidationRequest.
 * @param request - The noir NullifierKeyValidationRequest.
 * @returns The TS NullifierKeyValidationRequest.
 */
export function mapNullifierKeyValidationRequestFromNoir(
  request: NullifierKeyValidationRequestNoir,
): NullifierKeyValidationRequest {
  return new NullifierKeyValidationRequest(
    mapPointFromNoir(request.public_key),
    mapGrumpkinPrivateKeyFromNoir(request.secret_key),
  );
}

/**
 * Maps a NullifierKeyValidationRequest to a noir NullifierKeyValidationRequest.
 * @param request - The NullifierKeyValidationRequest.
 * @returns The noir NullifierKeyValidationRequest.
 */
export function mapNullifierKeyValidationRequestContextToNoir(
  request: NullifierKeyValidationRequestContext,
): NullifierKeyValidationRequestContextNoir {
  return {
    public_key: mapPointToNoir(request.publicKey),
    secret_key: mapGrumpkinPrivateKeyToNoir(request.secretKey),
    contract_address: mapAztecAddressToNoir(request.contractAddress),
  };
}

/**
 * Maps a noir NullifierKeyValidationRequestContext to NullifierKeyValidationRequestContext.
 * @param request - The noir NullifierKeyValidationRequestContext.
 * @returns The TS NullifierKeyValidationRequestContext.
 */
export function mapNullifierKeyValidationRequestContextFromNoir(
  request: NullifierKeyValidationRequestContextNoir,
): NullifierKeyValidationRequestContext {
  return new NullifierKeyValidationRequestContext(
    mapPointFromNoir(request.public_key),
    mapGrumpkinPrivateKeyFromNoir(request.secret_key),
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
    return_values: mapTuple(privateCircuitPublicInputs.returnValues, mapFieldToNoir),
    note_hash_read_requests: mapTuple(privateCircuitPublicInputs.noteHashReadRequests, mapSideEffectToNoir),
    nullifier_read_requests: mapTuple(privateCircuitPublicInputs.nullifierReadRequests, mapReadRequestToNoir),
    nullifier_key_validation_requests: mapTuple(
      privateCircuitPublicInputs.nullifierKeyValidationRequests,
      mapNullifierKeyValidationRequestToNoir,
    ),
    new_note_hashes: mapTuple(privateCircuitPublicInputs.newNoteHashes, mapSideEffectToNoir),
    new_nullifiers: mapTuple(privateCircuitPublicInputs.newNullifiers, mapSideEffectLinkedToNoir),
    private_call_stack_hashes: mapTuple(privateCircuitPublicInputs.privateCallStackHashes, mapFieldToNoir),
    public_call_stack_hashes: mapTuple(privateCircuitPublicInputs.publicCallStackHashes, mapFieldToNoir),
    new_l2_to_l1_msgs: mapTuple(privateCircuitPublicInputs.newL2ToL1Msgs, mapL2ToL1MessageToNoir),
    start_side_effect_counter: mapFieldToNoir(privateCircuitPublicInputs.startSideEffectCounter),
    end_side_effect_counter: mapFieldToNoir(privateCircuitPublicInputs.endSideEffectCounter),
    encrypted_logs_hash: mapFieldToNoir(privateCircuitPublicInputs.encryptedLogsHash),
    unencrypted_logs_hash: mapFieldToNoir(privateCircuitPublicInputs.unencryptedLogsHash),
    encrypted_log_preimages_length: mapFieldToNoir(privateCircuitPublicInputs.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: mapFieldToNoir(privateCircuitPublicInputs.unencryptedLogPreimagesLength),
    historical_header: mapHeaderToNoir(privateCircuitPublicInputs.historicalHeader),
    chain_id: mapFieldToNoir(privateCircuitPublicInputs.chainId),
    version: mapFieldToNoir(privateCircuitPublicInputs.version),
    min_revertible_side_effect_counter: mapFieldToNoir(privateCircuitPublicInputs.minRevertibleSideEffectCounter),
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
 * Maps a function leaf membership witness to a noir function leaf membership witness.
 * @param membershipWitness - The membership witness.
 * @returns The noir function leaf membership witness.
 */
function mapFunctionLeafMembershipWitnessToNoir(
  membershipWitness: MembershipWitness<typeof FUNCTION_TREE_HEIGHT>,
): FunctionLeafMembershipWitnessNoir {
  return {
    leaf_index: membershipWitness.leafIndex.toString(),
    sibling_path: mapTuple(membershipWitness.siblingPath, mapFieldToNoir),
  };
}

/**
 * Maps a read request membership witness to a noir read request membership witness.
 * @param noteHashReadRequestMembershipWitness - The read request membership witness.
 * @returns The noir read request membership witness.
 */
export function mapNoteHashReadRequestMembershipWitnessToNoir(
  noteHashReadRequestMembershipWitness: NoteHashReadRequestMembershipWitness,
): NoteHashReadRequestMembershipWitnessNoir {
  return {
    leaf_index: mapFieldToNoir(noteHashReadRequestMembershipWitness.leafIndex),
    sibling_path: mapTuple(noteHashReadRequestMembershipWitness.siblingPath, mapFieldToNoir),
    is_transient: noteHashReadRequestMembershipWitness.isTransient,
    hint_to_note_hash: mapFieldToNoir(noteHashReadRequestMembershipWitness.hintToNoteHash),
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
    private_call_stack: mapTuple(privateCallData.privateCallStack, mapCallRequestToNoir),
    public_call_stack: mapTuple(privateCallData.publicCallStack, mapCallRequestToNoir),
    proof: {},
    vk: {},
    function_leaf_membership_witness: mapFunctionLeafMembershipWitnessToNoir(
      privateCallData.functionLeafMembershipWitness,
    ),
    note_hash_read_request_membership_witnesses: mapTuple(
      privateCallData.noteHashReadRequestMembershipWitnesses,
      mapNoteHashReadRequestMembershipWitnessToNoir,
    ),
    contract_class_artifact_hash: mapFieldToNoir(privateCallData.contractClassArtifactHash),
    contract_class_public_bytecode_commitment: mapFieldToNoir(privateCallData.contractClassPublicBytecodeCommitment),
    public_keys_hash: mapWrappedFieldToNoir(privateCallData.publicKeysHash),
    salted_initialization_hash: mapWrappedFieldToNoir(privateCallData.saltedInitializationHash),
    //TODO this seems like the wrong type in circuits.js
    portal_contract_address: mapEthAddressToNoir(EthAddress.fromField(privateCallData.portalContractAddress)),
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

function mapNullifierSettledReadHintToNoir(
  hint: SettledReadHint<typeof NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage>,
): NullifierSettledReadHintNoir {
  return {
    read_request_index: mapNumberToNoir(hint.readRequestIndex),
    membership_witness: mapNullifierMembershipWitnessToNoir(hint.membershipWitness),
    leaf_preimage: mapNullifierLeafPreimageToNoir(hint.leafPreimage),
  };
}

function mapNullifierReadRequestHintsToNoir(hints: NullifierReadRequestHints): NullifierReadRequestHintsNoir {
  return {
    read_request_statuses: mapTuple(hints.readRequestStatuses, mapReadRequestStatusToNoir),
    pending_read_hints: mapTuple(hints.pendingReadHints, mapPendingReadHintToNoir),
    settled_read_hints: mapTuple(hints.settledReadHints, mapNullifierSettledReadHintToNoir),
  };
}

function mapNullifierNonMembershipHintToNoir(
  hint: NonMembershipHint<typeof NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage>,
): NullifierNonMembershipHintNoir {
  return {
    low_leaf_preimage: mapNullifierLeafPreimageToNoir(hint.leafPreimage),
    membership_witness: mapNullifierMembershipWitnessToNoir(hint.membershipWitness),
  };
}

function mapNullifierNonExistentReadRequestHintsToNoir(
  hints: NullifierNonExistentReadRequestHints,
): NullifierNonExistentReadRequestHintsNoir {
  return {
    non_membership_hints: mapTuple(hints.nonMembershipHints, mapNullifierNonMembershipHintToNoir),
    sorted_pending_values: mapTuple(hints.sortedPendingValues, mapSideEffectLinkedToNoir),
    sorted_pending_value_index_hints: mapTuple(hints.sortedPendingValueHints, mapNumberToNoir),
    next_pending_value_indices: mapTuple(hints.nextPendingValueIndices, mapNumberToNoir),
  };
}

function mapValidationRequestsToNoir(requests: ValidationRequests): ValidationRequestsNoir {
  return {
    for_rollup: mapRollupValidationRequestsToNoir(requests.forRollup),
    note_hash_read_requests: mapTuple(requests.noteHashReadRequests, mapSideEffectToNoir),
    nullifier_read_requests: mapTuple(requests.nullifierReadRequests, mapReadRequestContextToNoir),
    nullifier_non_existent_read_requests: mapTuple(
      requests.nullifierNonExistentReadRequests,
      mapReadRequestContextToNoir,
    ),
    nullifier_key_validation_requests: mapTuple(
      requests.nullifierKeyValidationRequests,
      mapNullifierKeyValidationRequestContextToNoir,
    ),
    public_data_reads: mapTuple(requests.publicDataReads, mapPublicDataReadToNoir),
  };
}

function mapValidationRequestsFromNoir(requests: ValidationRequestsNoir): ValidationRequests {
  return new ValidationRequests(
    mapRollupValidationRequestsFromNoir(requests.for_rollup),
    mapTupleFromNoir(requests.note_hash_read_requests, MAX_NOTE_HASH_READ_REQUESTS_PER_TX, mapSideEffectFromNoir),
    mapTupleFromNoir(
      requests.nullifier_read_requests,
      MAX_NULLIFIER_READ_REQUESTS_PER_TX,
      mapReadRequestContextFromNoir,
    ),
    mapTupleFromNoir(
      requests.nullifier_non_existent_read_requests,
      MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
      mapReadRequestContextFromNoir,
    ),
    mapTupleFromNoir(
      requests.nullifier_key_validation_requests,
      MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
      mapNullifierKeyValidationRequestContextFromNoir,
    ),
    mapTupleFromNoir(requests.public_data_reads, MAX_PUBLIC_DATA_READS_PER_TX, mapPublicDataReadFromNoir),
  );
}

export function mapPrivateAccumulatedDataFromNoir(
  privateAccumulatedData: PrivateAccumulatedDataNoir,
): PrivateAccumulatedData {
  return new PrivateAccumulatedData(
    mapTupleFromNoir(privateAccumulatedData.new_note_hashes, MAX_NEW_NOTE_HASHES_PER_TX, mapSideEffectFromNoir),
    mapTupleFromNoir(privateAccumulatedData.new_nullifiers, MAX_NEW_NULLIFIERS_PER_TX, mapSideEffectLinkedFromNoir),
    mapTupleFromNoir(privateAccumulatedData.new_l2_to_l1_msgs, MAX_NEW_L2_TO_L1_MSGS_PER_TX, mapFieldFromNoir),
    mapFieldFromNoir(privateAccumulatedData.encrypted_logs_hash),
    mapFieldFromNoir(privateAccumulatedData.unencrypted_logs_hash),
    mapFieldFromNoir(privateAccumulatedData.encrypted_log_preimages_length),
    mapFieldFromNoir(privateAccumulatedData.unencrypted_log_preimages_length),
    mapTupleFromNoir(
      privateAccumulatedData.private_call_stack,
      MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
      mapCallRequestFromNoir,
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
    new_note_hashes: mapTuple(data.newNoteHashes, mapSideEffectToNoir),
    new_nullifiers: mapTuple(data.newNullifiers, mapSideEffectLinkedToNoir),
    new_l2_to_l1_msgs: mapTuple(data.newL2ToL1Msgs, mapFieldToNoir),
    encrypted_logs_hash: mapFieldToNoir(data.encryptedLogsHash),
    unencrypted_logs_hash: mapFieldToNoir(data.unencryptedLogsHash),
    encrypted_log_preimages_length: mapFieldToNoir(data.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: mapFieldToNoir(data.unencryptedLogPreimagesLength),
    private_call_stack: mapTuple(data.privateCallStack, mapCallRequestToNoir),
    public_call_stack: mapTuple(data.publicCallStack, mapCallRequestToNoir),
  };
}

export function mapPublicAccumulatedDataFromNoir(
  publicAccumulatedData: PublicAccumulatedDataNoir,
): PublicAccumulatedData {
  return new PublicAccumulatedData(
    mapTupleFromNoir(publicAccumulatedData.new_note_hashes, MAX_NEW_NOTE_HASHES_PER_TX, mapSideEffectFromNoir),
    mapTupleFromNoir(publicAccumulatedData.new_nullifiers, MAX_NEW_NULLIFIERS_PER_TX, mapSideEffectLinkedFromNoir),
    mapTupleFromNoir(publicAccumulatedData.new_l2_to_l1_msgs, MAX_NEW_L2_TO_L1_MSGS_PER_TX, mapFieldFromNoir),
    mapFieldFromNoir(publicAccumulatedData.encrypted_logs_hash),
    mapFieldFromNoir(publicAccumulatedData.unencrypted_logs_hash),
    mapFieldFromNoir(publicAccumulatedData.encrypted_log_preimages_length),
    mapFieldFromNoir(publicAccumulatedData.unencrypted_log_preimages_length),
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
  );
}

export function mapPublicAccumulatedDataToNoir(
  publicAccumulatedData: PublicAccumulatedData,
): PublicAccumulatedDataNoir {
  return {
    new_note_hashes: mapTuple(publicAccumulatedData.newNoteHashes, mapSideEffectToNoir),
    new_nullifiers: mapTuple(publicAccumulatedData.newNullifiers, mapSideEffectLinkedToNoir),
    new_l2_to_l1_msgs: mapTuple(publicAccumulatedData.newL2ToL1Msgs, mapFieldToNoir),
    encrypted_logs_hash: mapFieldToNoir(publicAccumulatedData.encryptedLogsHash),
    unencrypted_logs_hash: mapFieldToNoir(publicAccumulatedData.unencryptedLogsHash),
    encrypted_log_preimages_length: mapFieldToNoir(publicAccumulatedData.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: mapFieldToNoir(publicAccumulatedData.unencryptedLogPreimagesLength),
    public_data_update_requests: mapTuple(
      publicAccumulatedData.publicDataUpdateRequests,
      mapPublicDataUpdateRequestToNoir,
    ),
    public_call_stack: mapTuple(publicAccumulatedData.publicCallStack, mapCallRequestToNoir),
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
    mapTupleFromNoir(combinedAccumulatedData.new_note_hashes, MAX_NEW_NOTE_HASHES_PER_TX, mapSideEffectFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.new_nullifiers, MAX_NEW_NULLIFIERS_PER_TX, mapSideEffectLinkedFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.new_l2_to_l1_msgs, MAX_NEW_L2_TO_L1_MSGS_PER_TX, mapFieldFromNoir),
    mapFieldFromNoir(combinedAccumulatedData.encrypted_logs_hash),
    mapFieldFromNoir(combinedAccumulatedData.unencrypted_logs_hash),
    mapFieldFromNoir(combinedAccumulatedData.encrypted_log_preimages_length),
    mapFieldFromNoir(combinedAccumulatedData.unencrypted_log_preimages_length),
    mapTupleFromNoir(
      combinedAccumulatedData.public_data_update_requests,
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      mapPublicDataUpdateRequestFromNoir,
    ),
  );
}

export function mapCombinedAccumulatedDataToNoir(
  combinedAccumulatedData: CombinedAccumulatedData,
): CombinedAccumulatedDataNoir {
  return {
    new_note_hashes: mapTuple(combinedAccumulatedData.newNoteHashes, mapSideEffectToNoir),
    new_nullifiers: mapTuple(combinedAccumulatedData.newNullifiers, mapSideEffectLinkedToNoir),
    new_l2_to_l1_msgs: mapTuple(combinedAccumulatedData.newL2ToL1Msgs, mapFieldToNoir),
    encrypted_logs_hash: mapFieldToNoir(combinedAccumulatedData.encryptedLogsHash),
    unencrypted_logs_hash: mapFieldToNoir(combinedAccumulatedData.unencryptedLogsHash),
    encrypted_log_preimages_length: mapFieldToNoir(combinedAccumulatedData.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: mapFieldToNoir(combinedAccumulatedData.unencryptedLogPreimagesLength),
    public_data_update_requests: mapTuple(
      combinedAccumulatedData.publicDataUpdateRequests,
      mapPublicDataUpdateRequestToNoir,
    ),
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
  };
}

export function mapPublicKernelCircuitPublicInputsToNoir(
  inputs: PublicKernelCircuitPublicInputs,
): PublicKernelCircuitPublicInputsNoir {
  return {
    aggregation_object: {},
    constants: mapCombinedConstantDataToNoir(inputs.constants),
    validation_requests: mapValidationRequestsToNoir(inputs.validationRequests),
    end: mapPublicAccumulatedDataToNoir(inputs.end),
    end_non_revertible: mapPublicAccumulatedDataToNoir(inputs.endNonRevertibleData),
    revert_code: mapRevertCodeToNoir(inputs.revertCode),
  };
}

export function mapKernelCircuitPublicInputsFromNoir(inputs: KernelCircuitPublicInputsNoir) {
  return new KernelCircuitPublicInputs(
    AggregationObject.makeFake(),
    mapRollupValidationRequestsFromNoir(inputs.rollup_validation_requests),
    mapCombinedAccumulatedDataFromNoir(inputs.end),
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapRevertCodeFromNoir(inputs.revert_code),
  );
}

export function mapKernelCircuitPublicInputsToNoir(inputs: KernelCircuitPublicInputs): KernelCircuitPublicInputsNoir {
  return {
    aggregation_object: {},
    rollup_validation_requests: mapRollupValidationRequestsToNoir(inputs.rollupValidationRequests),
    constants: mapCombinedConstantDataToNoir(inputs.constants),
    end: mapCombinedAccumulatedDataToNoir(inputs.end),
    revert_code: mapRevertCodeToNoir(inputs.revertCode),
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
    proof: {},
    vk: {},
    vk_index: mapFieldToNoir(new Fr(publicKernelData.vkIndex)),
    vk_path: mapTuple(publicKernelData.vkPath, mapFieldToNoir),
  };
}

export function mapKernelDataToNoir(kernelData: KernelData): KernelDataNoir {
  return {
    public_inputs: mapKernelCircuitPublicInputsToNoir(kernelData.publicInputs),
    proof: {},
    vk: {},
    vk_index: mapFieldToNoir(new Fr(kernelData.vkIndex)),
    vk_path: mapTuple(kernelData.vkPath, mapFieldToNoir),
  };
}

export function mapPrivateKernelCircuitPublicInputsFromNoir(
  inputs: PrivateKernelCircuitPublicInputsNoir,
): PrivateKernelCircuitPublicInputs {
  return new PrivateKernelCircuitPublicInputs(
    AggregationObject.makeFake(),
    mapFieldFromNoir(inputs.min_revertible_side_effect_counter),
    mapValidationRequestsFromNoir(inputs.validation_requests),
    mapPrivateAccumulatedDataFromNoir(inputs.end),
    mapCombinedConstantDataFromNoir(inputs.constants),
  );
}

export function mapPrivateKernelCircuitPublicInputsToNoir(
  inputs: PrivateKernelCircuitPublicInputs,
): PrivateKernelCircuitPublicInputsNoir {
  return {
    aggregation_object: {},
    constants: mapCombinedConstantDataToNoir(inputs.constants),
    validation_requests: mapValidationRequestsToNoir(inputs.validationRequests),
    end: mapPrivateAccumulatedDataToNoir(inputs.end),
    min_revertible_side_effect_counter: mapFieldToNoir(inputs.minRevertibleSideEffectCounter),
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
    proof: {},
    vk: {},
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
    AggregationObject.makeFake(),
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapRevertCodeFromNoir(inputs.revert_code),
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
  );
  return new PrivateKernelTailCircuitPublicInputs(
    AggregationObject.makeFake(),
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapRevertCodeFromNoir(inputs.revert_code),
    forPublic,
  );
}

export function mapPrivateKernelInitCircuitPrivateInputsToNoir(
  privateKernelInputsInit: PrivateKernelInitCircuitPrivateInputs,
): PrivateKernelInitCircuitPrivateInputsNoir {
  return {
    tx_request: mapTxRequestToNoir(privateKernelInputsInit.txRequest),
    private_call: mapPrivateCallDataToNoir(privateKernelInputsInit.privateCall),
  };
}

export function mapPrivateKernelInnerCircuitPrivateInputsToNoir(
  privateKernelInnerCircuitPrivateInputs: PrivateKernelInnerCircuitPrivateInputs,
): PrivateKernelInnerCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPrivateKernelDataToNoir(privateKernelInnerCircuitPrivateInputs.previousKernel),
    private_call: mapPrivateCallDataToNoir(privateKernelInnerCircuitPrivateInputs.privateCall),
  };
}

export function mapPrivateKernelTailCircuitPrivateInputsToNoir(
  inputs: PrivateKernelTailCircuitPrivateInputs,
): PrivateKernelTailCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPrivateKernelDataToNoir(inputs.previousKernel),
    sorted_new_note_hashes: mapTuple(inputs.sortedNewNoteHashes, mapSideEffectToNoir),
    sorted_new_note_hashes_indexes: mapTuple(inputs.sortedNewNoteHashesIndexes, mapNumberToNoir),
    read_commitment_hints: mapTuple(inputs.readCommitmentHints, mapFieldToNoir),
    sorted_new_nullifiers: mapTuple(inputs.sortedNewNullifiers, mapSideEffectLinkedToNoir),
    sorted_new_nullifiers_indexes: mapTuple(inputs.sortedNewNullifiersIndexes, mapNumberToNoir),
    nullifier_read_request_hints: mapNullifierReadRequestHintsToNoir(inputs.nullifierReadRequestHints),
    nullifier_commitment_hints: mapTuple(inputs.nullifierCommitmentHints, mapFieldToNoir),
    master_nullifier_secret_keys: mapTuple(inputs.masterNullifierSecretKeys, mapGrumpkinPrivateKeyToNoir),
  };
}

export function mapPrivateKernelTailToPublicCircuitPrivateInputsToNoir(
  inputs: PrivateKernelTailCircuitPrivateInputs,
): PrivateKernelTailToPublicCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPrivateKernelDataToNoir(inputs.previousKernel),
    sorted_new_note_hashes: mapTuple(inputs.sortedNewNoteHashes, mapSideEffectToNoir),
    sorted_new_note_hashes_indexes: mapTuple(inputs.sortedNewNoteHashesIndexes, mapNumberToNoir),
    read_commitment_hints: mapTuple(inputs.readCommitmentHints, mapFieldToNoir),
    sorted_new_nullifiers: mapTuple(inputs.sortedNewNullifiers, mapSideEffectLinkedToNoir),
    sorted_new_nullifiers_indexes: mapTuple(inputs.sortedNewNullifiersIndexes, mapNumberToNoir),
    nullifier_read_request_hints: mapNullifierReadRequestHintsToNoir(inputs.nullifierReadRequestHints),
    nullifier_commitment_hints: mapTuple(inputs.nullifierCommitmentHints, mapFieldToNoir),
    master_nullifier_secret_keys: mapTuple(inputs.masterNullifierSecretKeys, mapGrumpkinPrivateKeyToNoir),
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

export function mapPublicKernelTailCircuitPrivateInputsToNoir(
  inputs: PublicKernelTailCircuitPrivateInputs,
): PublicKernelTailCircuitPrivateInputsNoir {
  return {
    previous_kernel: mapPublicKernelDataToNoir(inputs.previousKernel),
    nullifier_read_request_hints: mapNullifierReadRequestHintsToNoir(inputs.nullifierReadRequestHints),
    nullifier_non_existent_read_request_hints: mapNullifierNonExistentReadRequestHintsToNoir(
      inputs.nullifierNonExistentReadRequestHints,
    ),
  };
}

export function mapPublicKernelCircuitPublicInputsFromNoir(
  inputs: PublicKernelCircuitPublicInputsNoir,
): PublicKernelCircuitPublicInputs {
  return new PublicKernelCircuitPublicInputs(
    AggregationObject.makeFake(),
    mapValidationRequestsFromNoir(inputs.validation_requests),
    mapPublicAccumulatedDataFromNoir(inputs.end_non_revertible),
    mapPublicAccumulatedDataFromNoir(inputs.end),
    mapCombinedConstantDataFromNoir(inputs.constants),
    mapRevertCodeFromNoir(inputs.revert_code),
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
  );
}

/**
 * Maps a constant rollup data to a noir constant rollup data.
 * @param constantRollupData - The circuits.js constant rollup data.
 * @returns The noir constant rollup data.
 */
export function mapConstantRollupDataToNoir(constantRollupData: ConstantRollupData): ConstantRollupDataNoir {
  return {
    last_archive: mapAppendOnlyTreeSnapshotToNoir(constantRollupData.lastArchive),
    private_kernel_vk_tree_root: mapFieldToNoir(constantRollupData.privateKernelVkTreeRoot),
    public_kernel_vk_tree_root: mapFieldToNoir(constantRollupData.publicKernelVkTreeRoot),
    base_rollup_vk_hash: mapFieldToNoir(constantRollupData.baseRollupVkHash),
    merge_rollup_vk_hash: mapFieldToNoir(constantRollupData.mergeRollupVkHash),
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
    return_values: mapTuple(publicInputs.returnValues, mapFieldToNoir),
    nullifier_read_requests: mapTuple(publicInputs.nullifierReadRequests, mapReadRequestToNoir),
    nullifier_non_existent_read_requests: mapTuple(publicInputs.nullifierNonExistentReadRequests, mapReadRequestToNoir),
    contract_storage_update_requests: mapTuple(
      publicInputs.contractStorageUpdateRequests,
      mapStorageUpdateRequestToNoir,
    ),
    contract_storage_reads: mapTuple(publicInputs.contractStorageReads, mapStorageReadToNoir),
    public_call_stack_hashes: mapTuple(publicInputs.publicCallStackHashes, mapFieldToNoir),
    new_note_hashes: mapTuple(publicInputs.newNoteHashes, mapSideEffectToNoir),
    new_nullifiers: mapTuple(publicInputs.newNullifiers, mapSideEffectLinkedToNoir),
    new_l2_to_l1_msgs: mapTuple(publicInputs.newL2ToL1Msgs, mapL2ToL1MessageToNoir),
    start_side_effect_counter: mapFieldToNoir(publicInputs.startSideEffectCounter),
    end_side_effect_counter: mapFieldToNoir(publicInputs.endSideEffectCounter),
    unencrypted_logs_hash: mapFieldToNoir(publicInputs.unencryptedLogsHash),
    unencrypted_log_preimages_length: mapFieldToNoir(publicInputs.unencryptedLogPreimagesLength),
    historical_header: mapHeaderToNoir(publicInputs.historicalHeader),

    prover_address: mapAztecAddressToNoir(publicInputs.proverAddress),
    revert_code: mapRevertCodeToNoir(publicInputs.revertCode),
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
    mapFieldFromNoir(constantRollupData.private_kernel_vk_tree_root),
    mapFieldFromNoir(constantRollupData.public_kernel_vk_tree_root),
    mapFieldFromNoir(constantRollupData.base_rollup_vk_hash),
    mapFieldFromNoir(constantRollupData.merge_rollup_vk_hash),
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
    height_in_block_tree: mapFieldToNoir(new Fr(baseOrMergeRollupPublicInputs.rollupSubtreeHeight)),
    aggregation_object: {},
    constants: mapConstantRollupDataToNoir(baseOrMergeRollupPublicInputs.constants),
    start: mapPartialStateReferenceToNoir(baseOrMergeRollupPublicInputs.start),
    end: mapPartialStateReferenceToNoir(baseOrMergeRollupPublicInputs.end),
    txs_effects_hash: mapFieldToNoir(baseOrMergeRollupPublicInputs.txsEffectsHash),
    out_hash: mapFieldToNoir(baseOrMergeRollupPublicInputs.outHash),
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
    portal_contract_address: mapEthAddressToNoir(EthAddress.fromField(publicCall.portalContractAddress)),
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
    mapFieldFromNoir(baseOrMergeRollupPublicInputs.height_in_block_tree),
    AggregationObject.makeFake(),
    mapConstantRollupDataFromNoir(baseOrMergeRollupPublicInputs.constants),
    mapPartialStateReferenceFromNoir(baseOrMergeRollupPublicInputs.start),
    mapPartialStateReferenceFromNoir(baseOrMergeRollupPublicInputs.end),
    mapFieldFromNoir(baseOrMergeRollupPublicInputs.txs_effects_hash),
    mapFieldFromNoir(baseOrMergeRollupPublicInputs.out_hash),
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
    proof: {},
    vk: {},
    vk_index: mapFieldToNoir(new Fr(previousRollupData.vkIndex)),
    vk_sibling_path: {
      leaf_index: mapFieldToNoir(new Fr(previousRollupData.vkSiblingPath.leafIndex)),
      sibling_path: mapTuple(previousRollupData.vkSiblingPath.siblingPath, mapFieldToNoir),
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

/**
 * Naos the root rollup inputs to noir.
 * @param rootRollupInputs - The circuits.js root rollup inputs.
 * @returns The noir root rollup inputs.
 */
export function mapRootRollupInputsToNoir(rootRollupInputs: RootRollupInputs): RootRollupInputsNoir {
  return {
    previous_rollup_data: mapTuple(rootRollupInputs.previousRollupData, mapPreviousRollupDataToNoir),
    l1_to_l2_roots: mapRootParityInputToNoir(rootRollupInputs.l1ToL2Roots),
    new_l1_to_l2_messages: mapTuple(rootRollupInputs.newL1ToL2Messages, mapFieldToNoir),
    new_l1_to_l2_message_tree_root_sibling_path: mapTuple(
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

export function mapRootParityInputToNoir(rootParityInput: RootParityInput): RootParityInputNoir {
  return {
    proof: {},
    public_inputs: mapParityPublicInputsToNoir(rootParityInput.publicInputs),
  };
}

export function mapParityPublicInputsToNoir(parityPublicInputs: ParityPublicInputs): ParityPublicInputsNoir {
  return {
    aggregation_object: {},
    sha_root: mapFieldToNoir(parityPublicInputs.shaRoot),
    converted_root: mapFieldToNoir(parityPublicInputs.convertedRoot),
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
    AggregationObject.makeFake(),
    mapAppendOnlyTreeSnapshotFromNoir(rootRollupPublicInputs.archive),
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
    AggregationObject.makeFake(),
    mapFieldFromNoir(parityPublicInputs.sha_root),
    mapFieldFromNoir(parityPublicInputs.converted_root),
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
  );
}

/**
 * Maps a content commitment to Noir
 *
 */
export function mapContentCommitmentToNoir(contentCommitment: ContentCommitment): ContentCommitmentNoir {
  return {
    tx_tree_height: mapFieldToNoir(contentCommitment.txTreeHeight),
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
    mapFieldFromNoir(contentCommitment.tx_tree_height),
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

/**
 * Maps a nullifier membership witness to noir.
 * @param membershipWitness - The nullifier membership witness.
 * @returns The noir nullifier membership witness.
 */
export function mapNullifierMembershipWitnessToNoir(
  membershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>,
): NullifierMembershipWitnessNoir {
  return {
    leaf_index: membershipWitness.leafIndex.toString(),
    sibling_path: mapTuple(membershipWitness.siblingPath, mapFieldToNoir),
  };
}

/**
 * Maps a membership witness of the public data tree to noir.
 */
export function mapPublicDataMembershipWitnessToNoir(
  membershipWitness: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
): PublicDataMembershipWitnessNoir {
  return {
    leaf_index: membershipWitness.leafIndex.toString(),
    sibling_path: mapTuple(membershipWitness.siblingPath, mapFieldToNoir),
  };
}

/**
 * Maps a membership witness of the blocks tree to noir.
 * @param membershipWitness - The membership witness.
 * @returns The noir membership witness.
 */
export function mapArchiveRootMembershipWitnessToNoir(
  membershipWitness: MembershipWitness<typeof ARCHIVE_HEIGHT>,
): ArchiveRootMembershipWitnessNoir {
  return {
    leaf_index: membershipWitness.leafIndex.toString(),
    sibling_path: mapTuple(membershipWitness.siblingPath, mapFieldToNoir),
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
      mapNullifierMembershipWitnessToNoir,
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
      mapPublicDataMembershipWitnessToNoir,
    ),

    archive_root_membership_witness: mapArchiveRootMembershipWitnessToNoir(inputs.archiveRootMembershipWitness),
    constants: mapConstantRollupDataToNoir(inputs.constants),
  };
}

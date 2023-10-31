import {
  AggregationObject,
  AztecAddress,
  CallContext,
  CombinedAccumulatedData,
  CombinedConstantData,
  ContractDeploymentData,
  EthAddress,
  FinalAccumulatedData,
  Fr,
  FunctionData,
  FunctionSelector,
  HistoricBlockData,
  KernelCircuitPublicInputs,
  KernelCircuitPublicInputsFinal,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX,
  MAX_PENDING_READ_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_READ_REQUESTS_PER_TX,
  MembershipWitness,
  NewContractData,
  OptionallyRevealedData,
  Point,
  PreviousKernelData,
  PrivateCallData,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelInputsOrdering,
  PublicDataRead,
  PublicDataUpdateRequest,
  ReadRequestMembershipWitness,
  TxContext,
  TxRequest,
} from '@aztec/circuits.js';
import { Tuple } from '@aztec/foundation/serialize';

import {
  CallContext as CallContextNoir,
  CombinedAccumulatedData as CombinedAccumulatedDataNoir,
  CombinedConstantData as CombinedConstantDataNoir,
  ContractDeploymentData as ContractDeploymentDataNoir,
  ContractLeafMembershipWitness as ContractLeafMembershipWitnessNoir,
  FixedLengthArray,
  FunctionData as FunctionDataNoir,
  FunctionLeafMembershipWitness as FunctionLeafMembershipWitnessNoir,
  FunctionSelector as FunctionSelectorNoir,
  HistoricalBlockData as HistoricalBlockDataNoir,
  KernelCircuitPublicInputs as KernelCircuitPublicInputsNoir,
  NewContractData as NewContractDataNoir,
  Address as NoirAztecAddress,
  EthAddress as NoirEthAddress,
  Field as NoirField,
  Point as NoirPoint,
  OptionallyRevealedData as OptionallyRevealedDataNoir,
  PrivateCallData as PrivateCallDataNoir,
  PrivateCallStackItem as PrivateCallStackItemNoir,
  PrivateCircuitPublicInputs as PrivateCircuitPublicInputsNoir,
  PrivateKernelInputsInit as PrivateKernelInputsInitNoir,
  PublicDataRead as PublicDataReadNoir,
  PublicDataUpdateRequest as PublicDataUpdateRequestNoir,
  ReadRequestMembershipWitness as ReadRequestMembershipWitnessNoir,
  TxContext as TxContextNoir,
  TxRequest as TxRequestNoir,
} from './types/private_kernel_init_types.js';
import {
  PreviousKernelData as PreviousKernelDataNoir,
  PrivateKernelInputsInner as PrivateKernelInputsInnerNoir,
} from './types/private_kernel_inner_types.js';
import {
  FinalAccumulatedData as FinalAccumulatedDataNoir,
  KernelCircuitPublicInputsFinal as KernelCircuitPublicInputsFinalNoir,
  PrivateKernelInputsOrdering as PrivateKernelInputsOrderingNoir,
} from './types/private_kernel_ordering_types.js';

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
 * Maps a contract deployment data to a noir contract deployment data.
 * @param data - The data.
 * @returns The noir contract deployment data.
 */
export function mapContractDeploymentDataToNoir(data: ContractDeploymentData): ContractDeploymentDataNoir {
  return {
    deployer_public_key: mapPointToNoir(data.deployerPublicKey),
    constructor_vk_hash: mapFieldToNoir(data.constructorVkHash),
    function_tree_root: mapFieldToNoir(data.functionTreeRoot),
    contract_address_salt: mapFieldToNoir(data.contractAddressSalt),
    portal_contract_address: mapEthAddressToNoir(data.portalContractAddress),
  };
}

/**
 * Maps a noir contract deployment data to a contract deployment data.
 * @param data - The noir data.
 * @returns The contract deployment data.
 */
export function mapContractDeploymentDataFromNoir(data: ContractDeploymentDataNoir): ContractDeploymentData {
  return new ContractDeploymentData(
    mapPointFromNoir(data.deployer_public_key),
    mapFieldFromNoir(data.constructor_vk_hash),
    mapFieldFromNoir(data.function_tree_root),
    mapFieldFromNoir(data.contract_address_salt),
    mapEthAddressFromNoir(data.portal_contract_address),
  );
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
    is_contract_deployment_tx: txContext.isContractDeploymentTx,
    contract_deployment_data: mapContractDeploymentDataToNoir(txContext.contractDeploymentData),
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
    txContext.is_contract_deployment_tx,
    mapContractDeploymentDataFromNoir(txContext.contract_deployment_data),
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
    is_internal: functionData.isInternal,
    is_private: functionData.isPrivate,
    is_constructor: functionData.isConstructor,
  };
}

/**
 * Maps a noir function data to a function data.
 * @param functionData - The noir function data.
 * @returns The function data.
 */
export function mapFunctionDataFromNoir(functionData: FunctionDataNoir): FunctionData {
  return new FunctionData(
    mapFunctionSelectorFromNoir(functionData.selector),
    functionData.is_internal,
    functionData.is_private,
    functionData.is_constructor,
  );
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
export function mapCallContextToNoir(callContext: CallContext): CallContextNoir {
  return {
    msg_sender: mapAztecAddressToNoir(callContext.msgSender),
    storage_contract_address: mapAztecAddressToNoir(callContext.storageContractAddress),
    portal_contract_address: mapEthAddressToNoir(callContext.portalContractAddress),
    function_selector: mapFunctionSelectorToNoir(callContext.functionSelector),
    is_delegate_call: callContext.isDelegateCall,
    is_static_call: callContext.isStaticCall,
    is_contract_deployment: callContext.isContractDeployment,
  };
}

/**
 * Maps a historical block data to a noir historical block data.
 * @param historicalBlockData - The historical block data.
 * @returns The noir historical block data.
 */
export function mapHistoricalBlockDataToNoir(historicalBlockData: HistoricBlockData): HistoricalBlockDataNoir {
  return {
    blocks_tree_root: mapFieldToNoir(historicalBlockData.blocksTreeRoot),
    block: {
      note_hash_tree_root: mapFieldToNoir(historicalBlockData.noteHashTreeRoot),
      nullifier_tree_root: mapFieldToNoir(historicalBlockData.nullifierTreeRoot),
      contract_tree_root: mapFieldToNoir(historicalBlockData.contractTreeRoot),
      l1_to_l2_data_tree_root: mapFieldToNoir(historicalBlockData.l1ToL2MessagesTreeRoot),
      public_data_tree_root: mapFieldToNoir(historicalBlockData.publicDataTreeRoot),
      global_variables_hash: mapFieldToNoir(historicalBlockData.globalVariablesHash),
    },
    private_kernel_vk_tree_root: mapFieldToNoir(historicalBlockData.privateKernelVkTreeRoot),
  };
}

/**
 * Maps a noir historical block data to a historical block data.
 * @param historicalBlockData - The noir historical block data.
 * @returns The historical block data.
 */
export function mapHistoricalBlockDataFromNoir(historicalBlockData: HistoricalBlockDataNoir): HistoricBlockData {
  return new HistoricBlockData(
    mapFieldFromNoir(historicalBlockData.block.note_hash_tree_root),
    mapFieldFromNoir(historicalBlockData.block.nullifier_tree_root),
    mapFieldFromNoir(historicalBlockData.block.contract_tree_root),
    mapFieldFromNoir(historicalBlockData.block.l1_to_l2_data_tree_root),
    mapFieldFromNoir(historicalBlockData.blocks_tree_root),
    mapFieldFromNoir(historicalBlockData.private_kernel_vk_tree_root),
    mapFieldFromNoir(historicalBlockData.block.public_data_tree_root),
    mapFieldFromNoir(historicalBlockData.block.global_variables_hash),
  );
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
    call_context: mapCallContextToNoir(privateCircuitPublicInputs.callContext),
    args_hash: mapFieldToNoir(privateCircuitPublicInputs.argsHash),
    return_values: privateCircuitPublicInputs.returnValues.map(mapFieldToNoir) as FixedLengthArray<NoirField, 4>,
    read_requests: privateCircuitPublicInputs.readRequests.map(mapFieldToNoir) as FixedLengthArray<NoirField, 32>,
    pending_read_requests: privateCircuitPublicInputs.pendingReadRequests.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      32
    >,
    new_commitments: privateCircuitPublicInputs.newCommitments.map(mapFieldToNoir) as FixedLengthArray<NoirField, 16>,
    new_nullifiers: privateCircuitPublicInputs.newNullifiers.map(mapFieldToNoir) as FixedLengthArray<NoirField, 16>,
    nullified_commitments: privateCircuitPublicInputs.nullifiedCommitments.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      16
    >,
    private_call_stack: privateCircuitPublicInputs.privateCallStack.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      4
    >,
    public_call_stack: privateCircuitPublicInputs.publicCallStack.map(mapFieldToNoir) as FixedLengthArray<NoirField, 4>,
    new_l2_to_l1_msgs: privateCircuitPublicInputs.newL2ToL1Msgs.map(mapFieldToNoir) as FixedLengthArray<NoirField, 2>,
    encrypted_logs_hash: privateCircuitPublicInputs.encryptedLogsHash.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      2
    >,
    unencrypted_logs_hash: privateCircuitPublicInputs.unencryptedLogsHash.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      2
    >,
    encrypted_log_preimages_length: mapFieldToNoir(privateCircuitPublicInputs.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: mapFieldToNoir(privateCircuitPublicInputs.unencryptedLogPreimagesLength),
    historical_block_data: mapHistoricalBlockDataToNoir(privateCircuitPublicInputs.historicBlockData),
    contract_deployment_data: mapContractDeploymentDataToNoir(privateCircuitPublicInputs.contractDeploymentData),
    chain_id: mapFieldToNoir(privateCircuitPublicInputs.chainId),
    version: mapFieldToNoir(privateCircuitPublicInputs.version),
  };
}

/**
 * Maps a private call stack item to a noir private call stack item.
 * @param privateCallStackItem - The private call stack item.
 * @returns The noir private call stack item.
 */
export function mapPrivateCallStackItemToNoir(privateCallStackItem: PrivateCallStackItem): PrivateCallStackItemNoir {
  return {
    inner: {
      contract_address: mapAztecAddressToNoir(privateCallStackItem.contractAddress),
      public_inputs: mapPrivateCircuitPublicInputsToNoir(privateCallStackItem.publicInputs),
      is_execution_request: privateCallStackItem.isExecutionRequest,
      function_data: mapFunctionDataToNoir(privateCallStackItem.functionData),
    },
  };
}

/**
 * Maps a function leaf membership witness to a noir function leaf membership witness.
 * @param membershipWitness - The membership witness.
 * @returns The noir function leaf membership witness.
 */
function mapFunctionLeafMembershipWitnessToNoir(
  membershipWitness: MembershipWitness<4>,
): FunctionLeafMembershipWitnessNoir {
  return {
    leaf_index: membershipWitness.leafIndex.toString(),
    sibling_path: membershipWitness.siblingPath.map(mapFieldToNoir) as FixedLengthArray<NoirField, 4>,
  };
}

/**
 * Maps a contract leaf membership witness to a noir contract leaf membership witness.
 * @param membershipWitness - The membership witness.
 * @returns The noir contract leaf membership witness.
 */
function mapContractLeafMembershipWitnessToNoir(
  membershipWitness: MembershipWitness<16>,
): ContractLeafMembershipWitnessNoir {
  return {
    leaf_index: membershipWitness.leafIndex.toString(),
    sibling_path: membershipWitness.siblingPath.map(mapFieldToNoir) as FixedLengthArray<NoirField, 16>,
  };
}

/**
 * Maps a read request membership witness to a noir read request membership witness.
 * @param readRequestMembershipWitness - The read request membership witness.
 * @returns The noir read request membership witness.
 */
export function mapReadRequestMembershipWitnessToNoir(
  readRequestMembershipWitness: ReadRequestMembershipWitness,
): ReadRequestMembershipWitnessNoir {
  return {
    leaf_index: mapFieldToNoir(readRequestMembershipWitness.leafIndex),
    sibling_path: readRequestMembershipWitness.siblingPath.map(mapFieldToNoir) as FixedLengthArray<NoirField, 32>,
    is_transient: readRequestMembershipWitness.isTransient,
    hint_to_commitment: mapFieldToNoir(readRequestMembershipWitness.hintToCommitment),
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
    private_call_stack_preimages: privateCallData.privateCallStackPreimages.map(
      mapPrivateCallStackItemToNoir,
    ) as FixedLengthArray<PrivateCallStackItemNoir, 4>,
    proof: {},
    vk: {},
    function_leaf_membership_witness: mapFunctionLeafMembershipWitnessToNoir(
      privateCallData.functionLeafMembershipWitness,
    ),
    contract_leaf_membership_witness: mapContractLeafMembershipWitnessToNoir(
      privateCallData.contractLeafMembershipWitness,
    ),
    read_request_membership_witnesses: privateCallData.readRequestMembershipWitnesses.map(
      mapReadRequestMembershipWitnessToNoir,
    ) as FixedLengthArray<ReadRequestMembershipWitnessNoir, 32>,
    //TODO this seems like the wrong type in circuits.js
    portal_contract_address: mapEthAddressToNoir(EthAddress.fromField(privateCallData.portalContractAddress)),
    acir_hash: mapFieldToNoir(privateCallData.acirHash),
  };
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
 * Maps optionally revealed data from noir to the parsed type.
 * @param optionallyRevealedData - The noir optionally revealed data.
 * @returns The parsed optionally revealed data.
 */
export function mapOptionallyRevealedDataFromNoir(
  optionallyRevealedData: OptionallyRevealedDataNoir,
): OptionallyRevealedData {
  return new OptionallyRevealedData(
    mapFieldFromNoir(optionallyRevealedData.call_stack_item_hash),
    mapFunctionDataFromNoir(optionallyRevealedData.function_data),
    mapFieldFromNoir(optionallyRevealedData.vk_hash),
    mapEthAddressFromNoir(optionallyRevealedData.portal_contract_address),
    optionallyRevealedData.pay_fee_from_l1,
    optionallyRevealedData.pay_fee_from_public_l2,
    optionallyRevealedData.called_from_l1,
    optionallyRevealedData.called_from_public_l2,
  );
}

/**
 * Maps optionally revealed data to noir optionally revealed data.
 * @param optionallyRevealedData - The optionally revealed data.
 * @returns The noir optionally revealed data.
 */
export function mapOptionallyRevealedDataToNoir(
  optionallyRevealedData: OptionallyRevealedData,
): OptionallyRevealedDataNoir {
  return {
    call_stack_item_hash: mapFieldToNoir(optionallyRevealedData.callStackItemHash),
    function_data: mapFunctionDataToNoir(optionallyRevealedData.functionData),
    vk_hash: mapFieldToNoir(optionallyRevealedData.vkHash),
    portal_contract_address: mapEthAddressToNoir(optionallyRevealedData.portalContractAddress),
    pay_fee_from_l1: optionallyRevealedData.payFeeFromL1,
    pay_fee_from_public_l2: optionallyRevealedData.payFeeFromPublicL2,
    called_from_l1: optionallyRevealedData.calledFromL1,
    called_from_public_l2: optionallyRevealedData.calledFromPublicL2,
  };
}

/**
 * Maps new contract data from noir to the parsed type.
 * @param newContractData - The noir new contract data.
 * @returns The parsed new contract data.
 */
export function mapNewContractDataFromNoir(newContractData: NewContractDataNoir): NewContractData {
  return new NewContractData(
    mapAztecAddressFromNoir(newContractData.contract_address),
    mapEthAddressFromNoir(newContractData.portal_contract_address),
    mapFieldFromNoir(newContractData.function_tree_root),
  );
}

/**
 * Maps new contract data to noir new contract data.
 * @param newContractData - The new contract data.
 * @returns The noir new contract data.
 */
export function mapNewContractDataToNoir(newContractData: NewContractData): NewContractDataNoir {
  return {
    contract_address: mapAztecAddressToNoir(newContractData.contractAddress),
    portal_contract_address: mapEthAddressToNoir(newContractData.portalContractAddress),
    function_tree_root: mapFieldToNoir(newContractData.functionTreeRoot),
  };
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
    mapFieldFromNoir(publicDataUpdateRequest.leaf_index),
    mapFieldFromNoir(publicDataUpdateRequest.old_value),
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
    leaf_index: mapFieldToNoir(publicDataUpdateRequest.leafIndex),
    old_value: mapFieldToNoir(publicDataUpdateRequest.oldValue),
    new_value: mapFieldToNoir(publicDataUpdateRequest.newValue),
  };
}

/**
 * Maps public data read from noir to the parsed type.
 * @param publicDataRead - The noir public data read.
 * @returns The parsed public data read.
 */
export function mapPublicDataReadFromNoir(publicDataRead: PublicDataReadNoir): PublicDataRead {
  return new PublicDataRead(mapFieldFromNoir(publicDataRead.leaf_index), mapFieldFromNoir(publicDataRead.value));
}

/**
 * Maps public data read to noir public data read.
 * @param publicDataRead - The public data read.
 * @returns The noir public data read.
 */
export function mapPublicDataReadToNoir(publicDataRead: PublicDataRead): PublicDataReadNoir {
  return {
    leaf_index: mapFieldToNoir(publicDataRead.leafIndex),
    value: mapFieldToNoir(publicDataRead.value),
  };
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
    // TODO aggregation object
    AggregationObject.makeFake(),
    mapTupleFromNoir(combinedAccumulatedData.read_requests, MAX_READ_REQUESTS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.pending_read_requests, MAX_PENDING_READ_REQUESTS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.new_commitments, MAX_NEW_COMMITMENTS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.new_nullifiers, MAX_NEW_NULLIFIERS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.nullified_commitments, MAX_NEW_NULLIFIERS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(
      combinedAccumulatedData.private_call_stack,
      MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
      mapFieldFromNoir,
    ),
    mapTupleFromNoir(combinedAccumulatedData.public_call_stack, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.new_l2_to_l1_msgs, MAX_NEW_L2_TO_L1_MSGS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.encrypted_logs_hash, 2, mapFieldFromNoir),
    mapTupleFromNoir(combinedAccumulatedData.unencrypted_logs_hash, 2, mapFieldFromNoir),
    mapFieldFromNoir(combinedAccumulatedData.encrypted_log_preimages_length),
    mapFieldFromNoir(combinedAccumulatedData.unencrypted_log_preimages_length),
    mapTupleFromNoir(combinedAccumulatedData.new_contracts, MAX_NEW_CONTRACTS_PER_TX, mapNewContractDataFromNoir),
    mapTupleFromNoir(
      combinedAccumulatedData.optionally_revealed_data,
      MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX,
      mapOptionallyRevealedDataFromNoir,
    ),
    mapTupleFromNoir(
      combinedAccumulatedData.public_data_update_requests,
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      mapPublicDataUpdateRequestFromNoir,
    ),
    mapTupleFromNoir(
      combinedAccumulatedData.public_data_reads,
      MAX_PUBLIC_DATA_READS_PER_TX,
      mapPublicDataReadFromNoir,
    ),
  );
}

/**
 * Maps final accumulated data from noir to the parsed type.
 * @param finalAccumulatedData - The noir final accumulated data.
 * @returns The parsed final accumulated data.
 */
export function mapFinalAccumulatedDataFromNoir(finalAccumulatedData: FinalAccumulatedDataNoir): FinalAccumulatedData {
  return new FinalAccumulatedData(
    // TODO aggregation object
    AggregationObject.makeFake(),
    mapTupleFromNoir(finalAccumulatedData.new_commitments, MAX_NEW_COMMITMENTS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(finalAccumulatedData.new_nullifiers, MAX_NEW_NULLIFIERS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(finalAccumulatedData.nullified_commitments, MAX_NEW_NULLIFIERS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(finalAccumulatedData.private_call_stack, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(finalAccumulatedData.public_call_stack, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(finalAccumulatedData.new_l2_to_l1_msgs, MAX_NEW_L2_TO_L1_MSGS_PER_TX, mapFieldFromNoir),
    mapTupleFromNoir(finalAccumulatedData.encrypted_logs_hash, 2, mapFieldFromNoir),
    mapTupleFromNoir(finalAccumulatedData.unencrypted_logs_hash, 2, mapFieldFromNoir),
    mapFieldFromNoir(finalAccumulatedData.encrypted_log_preimages_length),
    mapFieldFromNoir(finalAccumulatedData.unencrypted_log_preimages_length),
    mapTupleFromNoir(finalAccumulatedData.new_contracts, MAX_NEW_CONTRACTS_PER_TX, mapNewContractDataFromNoir),
    mapTupleFromNoir(
      finalAccumulatedData.optionally_revealed_data,
      MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX,
      mapOptionallyRevealedDataFromNoir,
    ),
  );
}

/**
 * Maps combined accumulated data to noir combined accumulated data.
 * @param combinedAccumulatedData - The combined accumulated data.
 * @returns The noir combined accumulated data.
 */
export function mapCombinedAccumulatedDataToNoir(
  combinedAccumulatedData: CombinedAccumulatedData,
): CombinedAccumulatedDataNoir {
  return {
    aggregation_object: {},
    read_requests: combinedAccumulatedData.readRequests.map(mapFieldToNoir) as FixedLengthArray<NoirField, 128>,
    pending_read_requests: combinedAccumulatedData.pendingReadRequests.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      128
    >,
    new_commitments: combinedAccumulatedData.newCommitments.map(mapFieldToNoir) as FixedLengthArray<NoirField, 64>,
    new_nullifiers: combinedAccumulatedData.newNullifiers.map(mapFieldToNoir) as FixedLengthArray<NoirField, 64>,
    nullified_commitments: combinedAccumulatedData.nullifiedCommitments.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      64
    >,
    private_call_stack: combinedAccumulatedData.privateCallStack.map(mapFieldToNoir) as FixedLengthArray<NoirField, 8>,
    public_call_stack: combinedAccumulatedData.publicCallStack.map(mapFieldToNoir) as FixedLengthArray<NoirField, 8>,
    new_l2_to_l1_msgs: combinedAccumulatedData.newL2ToL1Msgs.map(mapFieldToNoir) as FixedLengthArray<NoirField, 2>,
    encrypted_logs_hash: combinedAccumulatedData.encryptedLogsHash.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      2
    >,
    unencrypted_logs_hash: combinedAccumulatedData.unencryptedLogsHash.map(mapFieldToNoir) as FixedLengthArray<
      NoirField,
      2
    >,
    encrypted_log_preimages_length: mapFieldToNoir(combinedAccumulatedData.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: mapFieldToNoir(combinedAccumulatedData.unencryptedLogPreimagesLength),
    new_contracts: combinedAccumulatedData.newContracts.map(mapNewContractDataToNoir) as FixedLengthArray<
      NewContractDataNoir,
      1
    >,
    optionally_revealed_data: combinedAccumulatedData.optionallyRevealedData.map(
      mapOptionallyRevealedDataToNoir,
    ) as FixedLengthArray<OptionallyRevealedDataNoir, 4>,
    public_data_update_requests: combinedAccumulatedData.publicDataUpdateRequests.map(
      mapPublicDataUpdateRequestToNoir,
    ) as FixedLengthArray<PublicDataUpdateRequestNoir, 16>,
    public_data_reads: combinedAccumulatedData.publicDataReads.map(mapPublicDataReadToNoir) as FixedLengthArray<
      PublicDataReadNoir,
      16
    >,
  };
}

/**
 * Maps combined constant data from noir to the parsed type.
 * @param combinedConstantData - The noir combined constant data.
 * @returns The parsed combined constant data.
 */
export function mapCombinedConstantDataFromNoir(combinedConstantData: CombinedConstantDataNoir): CombinedConstantData {
  return new CombinedConstantData(
    mapHistoricalBlockDataFromNoir(combinedConstantData.block_data),
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
    block_data: mapHistoricalBlockDataToNoir(combinedConstantData.blockData),
    tx_context: mapTxContextToNoir(combinedConstantData.txContext),
  };
}

/**
 * Maps the inputs to the private kernel init to the noir representation.
 * @param privateKernelInputsInit - The inputs to the private kernel init.
 * @returns The noir representation of those inputs.
 */
export function mapPrivateKernelInputsInitToNoir(
  privateKernelInputsInit: PrivateKernelInputsInit,
): PrivateKernelInputsInitNoir {
  return {
    tx_request: mapTxRequestToNoir(privateKernelInputsInit.txRequest),
    private_call: mapPrivateCallDataToNoir(privateKernelInputsInit.privateCall),
  };
}

/**
 * Maps a previous kernel data to a noir previous kernel data.
 * @param previousKernelData - The previous kernel data.
 * @returns The noir previous kernel data.
 */
export function mapPreviousKernelDataToNoir(previousKernelData: PreviousKernelData): PreviousKernelDataNoir {
  return {
    public_inputs: mapKernelCircuitPublicInputsToNoir(previousKernelData.publicInputs),
    proof: {},
    vk: {},
    vk_index: mapFieldToNoir(new Fr(previousKernelData.vkIndex)),
    vk_path: previousKernelData.vkPath.map(mapFieldToNoir) as FixedLengthArray<NoirField, 3>,
  };
}

/**
 * Maps the inputs to the private kernel inner to the noir representation.
 * @param privateKernelInputsInit - The inputs to the private kernel inner.
 * @returns The noir representation of those inputs.
 */
export function mapPrivateKernelInputsInnerToNoir(
  privateKernelInputsInit: PrivateKernelInputsInner,
): PrivateKernelInputsInnerNoir {
  return {
    previous_kernel: mapPreviousKernelDataToNoir(privateKernelInputsInit.previousKernel),
    private_call: mapPrivateCallDataToNoir(privateKernelInputsInit.privateCall),
  };
}

/**
 * Maps a private circuit public inputs from noir to the circuits.js type.
 * @param kernelCircuitPublicInputs - The noir private circuit public inputs.
 * @returns The circuits.js private circuit public inputs.
 */
export function mapKernelCircuitPublicInputsFromNoir(
  kernelCircuitPublicInputs: KernelCircuitPublicInputsNoir,
): KernelCircuitPublicInputs {
  return new KernelCircuitPublicInputs(
    mapCombinedAccumulatedDataFromNoir(kernelCircuitPublicInputs.end),
    mapCombinedConstantDataFromNoir(kernelCircuitPublicInputs.constants),
    kernelCircuitPublicInputs.is_private,
  );
}

/**
 * Maps a private kernel inputs init from the circuits.js type to noir.
 * @param publicInputs - The circuits.js private kernel inputs init.
 * @returns The noir private kernel inputs init.
 */
export function mapKernelCircuitPublicInputsToNoir(
  publicInputs: KernelCircuitPublicInputs,
): KernelCircuitPublicInputsNoir {
  return {
    end: mapCombinedAccumulatedDataToNoir(publicInputs.end),
    constants: mapCombinedConstantDataToNoir(publicInputs.constants),
    is_private: publicInputs.isPrivate,
  };
}

/**
 * Maps a private kernel inputs final from noir to the circuits.js type.
 * @param publicInputs - The noir private kernel inputs final.
 * @returns The circuits.js private kernel inputs final.
 */
export function mapKernelCircuitPublicInputsFinalFromNoir(
  publicInputs: KernelCircuitPublicInputsFinalNoir,
): KernelCircuitPublicInputsFinal {
  return new KernelCircuitPublicInputsFinal(
    mapFinalAccumulatedDataFromNoir(publicInputs.end),
    mapCombinedConstantDataFromNoir(publicInputs.constants),
    publicInputs.is_private,
  );
}

/**
 * Maps a private kernel inputs ordering from the circuits.js type to noir.
 * @param inputs - The circuits.js private kernel inputs ordering.
 * @returns The noir private kernel inputs ordering.
 */
export function mapPrivateKernelInputsOrderingToNoir(
  inputs: PrivateKernelInputsOrdering,
): PrivateKernelInputsOrderingNoir {
  return {
    previous_kernel: mapPreviousKernelDataToNoir(inputs.previousKernel),
    read_commitment_hints: inputs.readCommitmentHints.map(mapFieldToNoir) as FixedLengthArray<NoirField, 128>,
    nullifier_commitment_hints: inputs.nullifierCommitmentHints.map(mapFieldToNoir) as FixedLengthArray<NoirField, 64>,
  };
}

/* eslint-disable */
// GENERATED FILE DO NOT EDIT, RUN yarn remake-bindings
import { Tuple, mapTuple } from '@aztec/foundation/serialize';
import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';
import mapValues from 'lodash.mapvalues';

import { callCbind } from './cbind.js';
import {
  Address,
  AppendOnlyTreeSnapshot,
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CallContext,
  CircuitError,
  CombinedAccumulatedData,
  CombinedConstantData,
  CompleteAddress,
  ConstantRollupData,
  ContractDeploymentData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  FinalAccumulatedData,
  Fq,
  Fr,
  FunctionData,
  FunctionSelector,
  G1AffineElement,
  GlobalVariables,
  HistoricBlockData,
  KernelCircuitPublicInputs,
  KernelCircuitPublicInputsFinal,
  MembershipWitness4,
  MembershipWitness8,
  MembershipWitness16,
  MergeRollupInputs,
  NativeAggregationState,
  NewContractData,
  NullifierLeafPreimage,
  OptionallyRevealedData,
  Point,
  PreviousKernelData,
  PreviousRollupData,
  PrivateCallData,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelInputsOrdering,
  Proof,
  PublicCallData,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicDataRead,
  PublicDataUpdateRequest,
  PublicKernelInputs,
  ReadRequestMembershipWitness,
  RootRollupInputs,
  RootRollupPublicInputs,
  TxContext,
  TxRequest,
  VerificationKeyData,
  isCircuitError,
  toBuffer,
} from './types.js';

interface MsgpackPoint {
  x: Buffer;
  y: Buffer;
}

export function toPoint(o: MsgpackPoint): Point {
  if (o.x === undefined) {
    throw new Error('Expected x in Point deserialization');
  }
  if (o.y === undefined) {
    throw new Error('Expected y in Point deserialization');
  }
  return new Point(Fr.fromBuffer(o.x), Fr.fromBuffer(o.y));
}

export function fromPoint(o: Point): MsgpackPoint {
  if (o.x === undefined) {
    throw new Error('Expected x in Point serialization');
  }
  if (o.y === undefined) {
    throw new Error('Expected y in Point serialization');
  }
  return {
    x: toBuffer(o.x),
    y: toBuffer(o.y),
  };
}

interface MsgpackCompleteAddress {
  address: Buffer;
  public_key: MsgpackPoint;
  partial_address: Buffer;
}

export function toCompleteAddress(o: MsgpackCompleteAddress): CompleteAddress {
  if (o.address === undefined) {
    throw new Error('Expected address in CompleteAddress deserialization');
  }
  if (o.public_key === undefined) {
    throw new Error('Expected public_key in CompleteAddress deserialization');
  }
  if (o.partial_address === undefined) {
    throw new Error('Expected partial_address in CompleteAddress deserialization');
  }
  return new CompleteAddress(Address.fromBuffer(o.address), toPoint(o.public_key), Fr.fromBuffer(o.partial_address));
}

export function fromCompleteAddress(o: CompleteAddress): MsgpackCompleteAddress {
  if (o.address === undefined) {
    throw new Error('Expected address in CompleteAddress serialization');
  }
  if (o.publicKey === undefined) {
    throw new Error('Expected publicKey in CompleteAddress serialization');
  }
  if (o.partialAddress === undefined) {
    throw new Error('Expected partialAddress in CompleteAddress serialization');
  }
  return {
    address: toBuffer(o.address),
    public_key: fromPoint(o.publicKey),
    partial_address: toBuffer(o.partialAddress),
  };
}

interface MsgpackGlobalVariables {
  chain_id: Buffer;
  version: Buffer;
  block_number: Buffer;
  timestamp: Buffer;
}

export function toGlobalVariables(o: MsgpackGlobalVariables): GlobalVariables {
  if (o.chain_id === undefined) {
    throw new Error('Expected chain_id in GlobalVariables deserialization');
  }
  if (o.version === undefined) {
    throw new Error('Expected version in GlobalVariables deserialization');
  }
  if (o.block_number === undefined) {
    throw new Error('Expected block_number in GlobalVariables deserialization');
  }
  if (o.timestamp === undefined) {
    throw new Error('Expected timestamp in GlobalVariables deserialization');
  }
  return new GlobalVariables(
    Fr.fromBuffer(o.chain_id),
    Fr.fromBuffer(o.version),
    Fr.fromBuffer(o.block_number),
    Fr.fromBuffer(o.timestamp),
  );
}

export function fromGlobalVariables(o: GlobalVariables): MsgpackGlobalVariables {
  if (o.chainId === undefined) {
    throw new Error('Expected chainId in GlobalVariables serialization');
  }
  if (o.version === undefined) {
    throw new Error('Expected version in GlobalVariables serialization');
  }
  if (o.blockNumber === undefined) {
    throw new Error('Expected blockNumber in GlobalVariables serialization');
  }
  if (o.timestamp === undefined) {
    throw new Error('Expected timestamp in GlobalVariables serialization');
  }
  return {
    chain_id: toBuffer(o.chainId),
    version: toBuffer(o.version),
    block_number: toBuffer(o.blockNumber),
    timestamp: toBuffer(o.timestamp),
  };
}

interface MsgpackG1AffineElement {
  x: Buffer;
  y: Buffer;
}

export function toG1AffineElement(o: MsgpackG1AffineElement): G1AffineElement {
  if (o.x === undefined) {
    throw new Error('Expected x in G1AffineElement deserialization');
  }
  if (o.y === undefined) {
    throw new Error('Expected y in G1AffineElement deserialization');
  }
  return new G1AffineElement(Fq.fromBuffer(o.x), Fq.fromBuffer(o.y));
}

export function fromG1AffineElement(o: G1AffineElement): MsgpackG1AffineElement {
  if (o.x === undefined) {
    throw new Error('Expected x in G1AffineElement serialization');
  }
  if (o.y === undefined) {
    throw new Error('Expected y in G1AffineElement serialization');
  }
  return {
    x: toBuffer(o.x),
    y: toBuffer(o.y),
  };
}

interface MsgpackNativeAggregationState {
  P0: MsgpackG1AffineElement;
  P1: MsgpackG1AffineElement;
  public_inputs: Buffer[];
  proof_witness_indices: number[];
  has_data: boolean;
}

export function toNativeAggregationState(o: MsgpackNativeAggregationState): NativeAggregationState {
  if (o.P0 === undefined) {
    throw new Error('Expected P0 in NativeAggregationState deserialization');
  }
  if (o.P1 === undefined) {
    throw new Error('Expected P1 in NativeAggregationState deserialization');
  }
  if (o.public_inputs === undefined) {
    throw new Error('Expected public_inputs in NativeAggregationState deserialization');
  }
  if (o.proof_witness_indices === undefined) {
    throw new Error('Expected proof_witness_indices in NativeAggregationState deserialization');
  }
  if (o.has_data === undefined) {
    throw new Error('Expected has_data in NativeAggregationState deserialization');
  }
  return new NativeAggregationState(
    toG1AffineElement(o.P0),
    toG1AffineElement(o.P1),
    o.public_inputs.map((v: Buffer) => Fr.fromBuffer(v)),
    o.proof_witness_indices.map((v: number) => v),
    o.has_data,
  );
}

export function fromNativeAggregationState(o: NativeAggregationState): MsgpackNativeAggregationState {
  if (o.p0 === undefined) {
    throw new Error('Expected p0 in NativeAggregationState serialization');
  }
  if (o.p1 === undefined) {
    throw new Error('Expected p1 in NativeAggregationState serialization');
  }
  if (o.publicInputs === undefined) {
    throw new Error('Expected publicInputs in NativeAggregationState serialization');
  }
  if (o.proofWitnessIndices === undefined) {
    throw new Error('Expected proofWitnessIndices in NativeAggregationState serialization');
  }
  if (o.hasData === undefined) {
    throw new Error('Expected hasData in NativeAggregationState serialization');
  }
  return {
    P0: fromG1AffineElement(o.p0),
    P1: fromG1AffineElement(o.p1),
    public_inputs: o.publicInputs.map((v: Fr) => toBuffer(v)),
    proof_witness_indices: o.proofWitnessIndices.map((v: number) => v),
    has_data: o.hasData,
  };
}

interface MsgpackNewContractData {
  contract_address: Buffer;
  portal_contract_address: Buffer;
  function_tree_root: Buffer;
}

export function toNewContractData(o: MsgpackNewContractData): NewContractData {
  if (o.contract_address === undefined) {
    throw new Error('Expected contract_address in NewContractData deserialization');
  }
  if (o.portal_contract_address === undefined) {
    throw new Error('Expected portal_contract_address in NewContractData deserialization');
  }
  if (o.function_tree_root === undefined) {
    throw new Error('Expected function_tree_root in NewContractData deserialization');
  }
  return new NewContractData(
    Address.fromBuffer(o.contract_address),
    Address.fromBuffer(o.portal_contract_address),
    Fr.fromBuffer(o.function_tree_root),
  );
}

export function fromNewContractData(o: NewContractData): MsgpackNewContractData {
  if (o.contractAddress === undefined) {
    throw new Error('Expected contractAddress in NewContractData serialization');
  }
  if (o.portalContractAddress === undefined) {
    throw new Error('Expected portalContractAddress in NewContractData serialization');
  }
  if (o.functionTreeRoot === undefined) {
    throw new Error('Expected functionTreeRoot in NewContractData serialization');
  }
  return {
    contract_address: toBuffer(o.contractAddress),
    portal_contract_address: toBuffer(o.portalContractAddress),
    function_tree_root: toBuffer(o.functionTreeRoot),
  };
}

interface MsgpackFunctionSelector {
  value: number;
}

export function toFunctionSelector(o: MsgpackFunctionSelector): FunctionSelector {
  if (o.value === undefined) {
    throw new Error('Expected value in FunctionSelector deserialization');
  }
  return new FunctionSelector(o.value);
}

export function fromFunctionSelector(o: FunctionSelector): MsgpackFunctionSelector {
  if (o.value === undefined) {
    throw new Error('Expected value in FunctionSelector serialization');
  }
  return {
    value: o.value,
  };
}

interface MsgpackFunctionData {
  selector: MsgpackFunctionSelector;
  is_internal: boolean;
  is_private: boolean;
  is_constructor: boolean;
}

export function toFunctionData(o: MsgpackFunctionData): FunctionData {
  if (o.selector === undefined) {
    throw new Error('Expected selector in FunctionData deserialization');
  }
  if (o.is_internal === undefined) {
    throw new Error('Expected is_internal in FunctionData deserialization');
  }
  if (o.is_private === undefined) {
    throw new Error('Expected is_private in FunctionData deserialization');
  }
  if (o.is_constructor === undefined) {
    throw new Error('Expected is_constructor in FunctionData deserialization');
  }
  return new FunctionData(toFunctionSelector(o.selector), o.is_internal, o.is_private, o.is_constructor);
}

export function fromFunctionData(o: FunctionData): MsgpackFunctionData {
  if (o.selector === undefined) {
    throw new Error('Expected selector in FunctionData serialization');
  }
  if (o.isInternal === undefined) {
    throw new Error('Expected isInternal in FunctionData serialization');
  }
  if (o.isPrivate === undefined) {
    throw new Error('Expected isPrivate in FunctionData serialization');
  }
  if (o.isConstructor === undefined) {
    throw new Error('Expected isConstructor in FunctionData serialization');
  }
  return {
    selector: fromFunctionSelector(o.selector),
    is_internal: o.isInternal,
    is_private: o.isPrivate,
    is_constructor: o.isConstructor,
  };
}

interface MsgpackOptionallyRevealedData {
  call_stack_item_hash: Buffer;
  function_data: MsgpackFunctionData;
  vk_hash: Buffer;
  portal_contract_address: Buffer;
  pay_fee_from_l1: boolean;
  pay_fee_from_public_l2: boolean;
  called_from_l1: boolean;
  called_from_public_l2: boolean;
}

export function toOptionallyRevealedData(o: MsgpackOptionallyRevealedData): OptionallyRevealedData {
  if (o.call_stack_item_hash === undefined) {
    throw new Error('Expected call_stack_item_hash in OptionallyRevealedData deserialization');
  }
  if (o.function_data === undefined) {
    throw new Error('Expected function_data in OptionallyRevealedData deserialization');
  }
  if (o.vk_hash === undefined) {
    throw new Error('Expected vk_hash in OptionallyRevealedData deserialization');
  }
  if (o.portal_contract_address === undefined) {
    throw new Error('Expected portal_contract_address in OptionallyRevealedData deserialization');
  }
  if (o.pay_fee_from_l1 === undefined) {
    throw new Error('Expected pay_fee_from_l1 in OptionallyRevealedData deserialization');
  }
  if (o.pay_fee_from_public_l2 === undefined) {
    throw new Error('Expected pay_fee_from_public_l2 in OptionallyRevealedData deserialization');
  }
  if (o.called_from_l1 === undefined) {
    throw new Error('Expected called_from_l1 in OptionallyRevealedData deserialization');
  }
  if (o.called_from_public_l2 === undefined) {
    throw new Error('Expected called_from_public_l2 in OptionallyRevealedData deserialization');
  }
  return new OptionallyRevealedData(
    Fr.fromBuffer(o.call_stack_item_hash),
    toFunctionData(o.function_data),
    Fr.fromBuffer(o.vk_hash),
    Address.fromBuffer(o.portal_contract_address),
    o.pay_fee_from_l1,
    o.pay_fee_from_public_l2,
    o.called_from_l1,
    o.called_from_public_l2,
  );
}

export function fromOptionallyRevealedData(o: OptionallyRevealedData): MsgpackOptionallyRevealedData {
  if (o.callStackItemHash === undefined) {
    throw new Error('Expected callStackItemHash in OptionallyRevealedData serialization');
  }
  if (o.functionData === undefined) {
    throw new Error('Expected functionData in OptionallyRevealedData serialization');
  }
  if (o.vkHash === undefined) {
    throw new Error('Expected vkHash in OptionallyRevealedData serialization');
  }
  if (o.portalContractAddress === undefined) {
    throw new Error('Expected portalContractAddress in OptionallyRevealedData serialization');
  }
  if (o.payFeeFromL1 === undefined) {
    throw new Error('Expected payFeeFromL1 in OptionallyRevealedData serialization');
  }
  if (o.payFeeFromPublicL2 === undefined) {
    throw new Error('Expected payFeeFromPublicL2 in OptionallyRevealedData serialization');
  }
  if (o.calledFromL1 === undefined) {
    throw new Error('Expected calledFromL1 in OptionallyRevealedData serialization');
  }
  if (o.calledFromPublicL2 === undefined) {
    throw new Error('Expected calledFromPublicL2 in OptionallyRevealedData serialization');
  }
  return {
    call_stack_item_hash: toBuffer(o.callStackItemHash),
    function_data: fromFunctionData(o.functionData),
    vk_hash: toBuffer(o.vkHash),
    portal_contract_address: toBuffer(o.portalContractAddress),
    pay_fee_from_l1: o.payFeeFromL1,
    pay_fee_from_public_l2: o.payFeeFromPublicL2,
    called_from_l1: o.calledFromL1,
    called_from_public_l2: o.calledFromPublicL2,
  };
}

interface MsgpackPublicDataUpdateRequest {
  leaf_index: Buffer;
  old_value: Buffer;
  new_value: Buffer;
}

export function toPublicDataUpdateRequest(o: MsgpackPublicDataUpdateRequest): PublicDataUpdateRequest {
  if (o.leaf_index === undefined) {
    throw new Error('Expected leaf_index in PublicDataUpdateRequest deserialization');
  }
  if (o.old_value === undefined) {
    throw new Error('Expected old_value in PublicDataUpdateRequest deserialization');
  }
  if (o.new_value === undefined) {
    throw new Error('Expected new_value in PublicDataUpdateRequest deserialization');
  }
  return new PublicDataUpdateRequest(
    Fr.fromBuffer(o.leaf_index),
    Fr.fromBuffer(o.old_value),
    Fr.fromBuffer(o.new_value),
  );
}

export function fromPublicDataUpdateRequest(o: PublicDataUpdateRequest): MsgpackPublicDataUpdateRequest {
  if (o.leafIndex === undefined) {
    throw new Error('Expected leafIndex in PublicDataUpdateRequest serialization');
  }
  if (o.oldValue === undefined) {
    throw new Error('Expected oldValue in PublicDataUpdateRequest serialization');
  }
  if (o.newValue === undefined) {
    throw new Error('Expected newValue in PublicDataUpdateRequest serialization');
  }
  return {
    leaf_index: toBuffer(o.leafIndex),
    old_value: toBuffer(o.oldValue),
    new_value: toBuffer(o.newValue),
  };
}

interface MsgpackPublicDataRead {
  leaf_index: Buffer;
  value: Buffer;
}

export function toPublicDataRead(o: MsgpackPublicDataRead): PublicDataRead {
  if (o.leaf_index === undefined) {
    throw new Error('Expected leaf_index in PublicDataRead deserialization');
  }
  if (o.value === undefined) {
    throw new Error('Expected value in PublicDataRead deserialization');
  }
  return new PublicDataRead(Fr.fromBuffer(o.leaf_index), Fr.fromBuffer(o.value));
}

export function fromPublicDataRead(o: PublicDataRead): MsgpackPublicDataRead {
  if (o.leafIndex === undefined) {
    throw new Error('Expected leafIndex in PublicDataRead serialization');
  }
  if (o.value === undefined) {
    throw new Error('Expected value in PublicDataRead serialization');
  }
  return {
    leaf_index: toBuffer(o.leafIndex),
    value: toBuffer(o.value),
  };
}

interface MsgpackCombinedAccumulatedData {
  aggregation_object: MsgpackNativeAggregationState;
  read_requests: Tuple<Buffer, 128>;
  new_commitments: Tuple<Buffer, 64>;
  new_nullifiers: Tuple<Buffer, 64>;
  nullified_commitments: Tuple<Buffer, 64>;
  private_call_stack: Tuple<Buffer, 8>;
  public_call_stack: Tuple<Buffer, 8>;
  new_l2_to_l1_msgs: Tuple<Buffer, 2>;
  encrypted_logs_hash: Tuple<Buffer, 2>;
  unencrypted_logs_hash: Tuple<Buffer, 2>;
  encrypted_log_preimages_length: Buffer;
  unencrypted_log_preimages_length: Buffer;
  new_contracts: Tuple<MsgpackNewContractData, 1>;
  optionally_revealed_data: Tuple<MsgpackOptionallyRevealedData, 4>;
  public_data_update_requests: Tuple<MsgpackPublicDataUpdateRequest, 16>;
  public_data_reads: Tuple<MsgpackPublicDataRead, 16>;
}

export function toCombinedAccumulatedData(o: MsgpackCombinedAccumulatedData): CombinedAccumulatedData {
  if (o.aggregation_object === undefined) {
    throw new Error('Expected aggregation_object in CombinedAccumulatedData deserialization');
  }
  if (o.read_requests === undefined) {
    throw new Error('Expected read_requests in CombinedAccumulatedData deserialization');
  }
  if (o.new_commitments === undefined) {
    throw new Error('Expected new_commitments in CombinedAccumulatedData deserialization');
  }
  if (o.new_nullifiers === undefined) {
    throw new Error('Expected new_nullifiers in CombinedAccumulatedData deserialization');
  }
  if (o.nullified_commitments === undefined) {
    throw new Error('Expected nullified_commitments in CombinedAccumulatedData deserialization');
  }
  if (o.private_call_stack === undefined) {
    throw new Error('Expected private_call_stack in CombinedAccumulatedData deserialization');
  }
  if (o.public_call_stack === undefined) {
    throw new Error('Expected public_call_stack in CombinedAccumulatedData deserialization');
  }
  if (o.new_l2_to_l1_msgs === undefined) {
    throw new Error('Expected new_l2_to_l1_msgs in CombinedAccumulatedData deserialization');
  }
  if (o.encrypted_logs_hash === undefined) {
    throw new Error('Expected encrypted_logs_hash in CombinedAccumulatedData deserialization');
  }
  if (o.unencrypted_logs_hash === undefined) {
    throw new Error('Expected unencrypted_logs_hash in CombinedAccumulatedData deserialization');
  }
  if (o.encrypted_log_preimages_length === undefined) {
    throw new Error('Expected encrypted_log_preimages_length in CombinedAccumulatedData deserialization');
  }
  if (o.unencrypted_log_preimages_length === undefined) {
    throw new Error('Expected unencrypted_log_preimages_length in CombinedAccumulatedData deserialization');
  }
  if (o.new_contracts === undefined) {
    throw new Error('Expected new_contracts in CombinedAccumulatedData deserialization');
  }
  if (o.optionally_revealed_data === undefined) {
    throw new Error('Expected optionally_revealed_data in CombinedAccumulatedData deserialization');
  }
  if (o.public_data_update_requests === undefined) {
    throw new Error('Expected public_data_update_requests in CombinedAccumulatedData deserialization');
  }
  if (o.public_data_reads === undefined) {
    throw new Error('Expected public_data_reads in CombinedAccumulatedData deserialization');
  }
  return new CombinedAccumulatedData(
    toNativeAggregationState(o.aggregation_object),
    mapTuple(o.read_requests, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_commitments, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_nullifiers, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.nullified_commitments, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.private_call_stack, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.public_call_stack, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_l2_to_l1_msgs, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.encrypted_logs_hash, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.unencrypted_logs_hash, (v: Buffer) => Fr.fromBuffer(v)),
    Fr.fromBuffer(o.encrypted_log_preimages_length),
    Fr.fromBuffer(o.unencrypted_log_preimages_length),
    mapTuple(o.new_contracts, (v: MsgpackNewContractData) => toNewContractData(v)),
    mapTuple(o.optionally_revealed_data, (v: MsgpackOptionallyRevealedData) => toOptionallyRevealedData(v)),
    mapTuple(o.public_data_update_requests, (v: MsgpackPublicDataUpdateRequest) => toPublicDataUpdateRequest(v)),
    mapTuple(o.public_data_reads, (v: MsgpackPublicDataRead) => toPublicDataRead(v)),
  );
}

export function fromCombinedAccumulatedData(o: CombinedAccumulatedData): MsgpackCombinedAccumulatedData {
  if (o.aggregationObject === undefined) {
    throw new Error('Expected aggregationObject in CombinedAccumulatedData serialization');
  }
  if (o.readRequests === undefined) {
    throw new Error('Expected readRequests in CombinedAccumulatedData serialization');
  }
  if (o.newCommitments === undefined) {
    throw new Error('Expected newCommitments in CombinedAccumulatedData serialization');
  }
  if (o.newNullifiers === undefined) {
    throw new Error('Expected newNullifiers in CombinedAccumulatedData serialization');
  }
  if (o.nullifiedCommitments === undefined) {
    throw new Error('Expected nullifiedCommitments in CombinedAccumulatedData serialization');
  }
  if (o.privateCallStack === undefined) {
    throw new Error('Expected privateCallStack in CombinedAccumulatedData serialization');
  }
  if (o.publicCallStack === undefined) {
    throw new Error('Expected publicCallStack in CombinedAccumulatedData serialization');
  }
  if (o.newL2ToL1Msgs === undefined) {
    throw new Error('Expected newL2ToL1Msgs in CombinedAccumulatedData serialization');
  }
  if (o.encryptedLogsHash === undefined) {
    throw new Error('Expected encryptedLogsHash in CombinedAccumulatedData serialization');
  }
  if (o.unencryptedLogsHash === undefined) {
    throw new Error('Expected unencryptedLogsHash in CombinedAccumulatedData serialization');
  }
  if (o.encryptedLogPreimagesLength === undefined) {
    throw new Error('Expected encryptedLogPreimagesLength in CombinedAccumulatedData serialization');
  }
  if (o.unencryptedLogPreimagesLength === undefined) {
    throw new Error('Expected unencryptedLogPreimagesLength in CombinedAccumulatedData serialization');
  }
  if (o.newContracts === undefined) {
    throw new Error('Expected newContracts in CombinedAccumulatedData serialization');
  }
  if (o.optionallyRevealedData === undefined) {
    throw new Error('Expected optionallyRevealedData in CombinedAccumulatedData serialization');
  }
  if (o.publicDataUpdateRequests === undefined) {
    throw new Error('Expected publicDataUpdateRequests in CombinedAccumulatedData serialization');
  }
  if (o.publicDataReads === undefined) {
    throw new Error('Expected publicDataReads in CombinedAccumulatedData serialization');
  }
  return {
    aggregation_object: fromNativeAggregationState(o.aggregationObject),
    read_requests: mapTuple(o.readRequests, (v: Fr) => toBuffer(v)),
    new_commitments: mapTuple(o.newCommitments, (v: Fr) => toBuffer(v)),
    new_nullifiers: mapTuple(o.newNullifiers, (v: Fr) => toBuffer(v)),
    nullified_commitments: mapTuple(o.nullifiedCommitments, (v: Fr) => toBuffer(v)),
    private_call_stack: mapTuple(o.privateCallStack, (v: Fr) => toBuffer(v)),
    public_call_stack: mapTuple(o.publicCallStack, (v: Fr) => toBuffer(v)),
    new_l2_to_l1_msgs: mapTuple(o.newL2ToL1Msgs, (v: Fr) => toBuffer(v)),
    encrypted_logs_hash: mapTuple(o.encryptedLogsHash, (v: Fr) => toBuffer(v)),
    unencrypted_logs_hash: mapTuple(o.unencryptedLogsHash, (v: Fr) => toBuffer(v)),
    encrypted_log_preimages_length: toBuffer(o.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: toBuffer(o.unencryptedLogPreimagesLength),
    new_contracts: mapTuple(o.newContracts, (v: NewContractData) => fromNewContractData(v)),
    optionally_revealed_data: mapTuple(o.optionallyRevealedData, (v: OptionallyRevealedData) =>
      fromOptionallyRevealedData(v),
    ),
    public_data_update_requests: mapTuple(o.publicDataUpdateRequests, (v: PublicDataUpdateRequest) =>
      fromPublicDataUpdateRequest(v),
    ),
    public_data_reads: mapTuple(o.publicDataReads, (v: PublicDataRead) => fromPublicDataRead(v)),
  };
}

interface MsgpackHistoricBlockData {
  private_data_tree_root: Buffer;
  nullifier_tree_root: Buffer;
  contract_tree_root: Buffer;
  l1_to_l2_messages_tree_root: Buffer;
  blocks_tree_root: Buffer;
  private_kernel_vk_tree_root: Buffer;
  public_data_tree_root: Buffer;
  global_variables_hash: Buffer;
}

export function toHistoricBlockData(o: MsgpackHistoricBlockData): HistoricBlockData {
  if (o.private_data_tree_root === undefined) {
    throw new Error('Expected private_data_tree_root in HistoricBlockData deserialization');
  }
  if (o.nullifier_tree_root === undefined) {
    throw new Error('Expected nullifier_tree_root in HistoricBlockData deserialization');
  }
  if (o.contract_tree_root === undefined) {
    throw new Error('Expected contract_tree_root in HistoricBlockData deserialization');
  }
  if (o.l1_to_l2_messages_tree_root === undefined) {
    throw new Error('Expected l1_to_l2_messages_tree_root in HistoricBlockData deserialization');
  }
  if (o.blocks_tree_root === undefined) {
    throw new Error('Expected blocks_tree_root in HistoricBlockData deserialization');
  }
  if (o.private_kernel_vk_tree_root === undefined) {
    throw new Error('Expected private_kernel_vk_tree_root in HistoricBlockData deserialization');
  }
  if (o.public_data_tree_root === undefined) {
    throw new Error('Expected public_data_tree_root in HistoricBlockData deserialization');
  }
  if (o.global_variables_hash === undefined) {
    throw new Error('Expected global_variables_hash in HistoricBlockData deserialization');
  }
  return new HistoricBlockData(
    Fr.fromBuffer(o.private_data_tree_root),
    Fr.fromBuffer(o.nullifier_tree_root),
    Fr.fromBuffer(o.contract_tree_root),
    Fr.fromBuffer(o.l1_to_l2_messages_tree_root),
    Fr.fromBuffer(o.blocks_tree_root),
    Fr.fromBuffer(o.private_kernel_vk_tree_root),
    Fr.fromBuffer(o.public_data_tree_root),
    Fr.fromBuffer(o.global_variables_hash),
  );
}

export function fromHistoricBlockData(o: HistoricBlockData): MsgpackHistoricBlockData {
  if (o.privateDataTreeRoot === undefined) {
    throw new Error('Expected privateDataTreeRoot in HistoricBlockData serialization');
  }
  if (o.nullifierTreeRoot === undefined) {
    throw new Error('Expected nullifierTreeRoot in HistoricBlockData serialization');
  }
  if (o.contractTreeRoot === undefined) {
    throw new Error('Expected contractTreeRoot in HistoricBlockData serialization');
  }
  if (o.l1ToL2MessagesTreeRoot === undefined) {
    throw new Error('Expected l1ToL2MessagesTreeRoot in HistoricBlockData serialization');
  }
  if (o.blocksTreeRoot === undefined) {
    throw new Error('Expected blocksTreeRoot in HistoricBlockData serialization');
  }
  if (o.privateKernelVkTreeRoot === undefined) {
    throw new Error('Expected privateKernelVkTreeRoot in HistoricBlockData serialization');
  }
  if (o.publicDataTreeRoot === undefined) {
    throw new Error('Expected publicDataTreeRoot in HistoricBlockData serialization');
  }
  if (o.globalVariablesHash === undefined) {
    throw new Error('Expected globalVariablesHash in HistoricBlockData serialization');
  }
  return {
    private_data_tree_root: toBuffer(o.privateDataTreeRoot),
    nullifier_tree_root: toBuffer(o.nullifierTreeRoot),
    contract_tree_root: toBuffer(o.contractTreeRoot),
    l1_to_l2_messages_tree_root: toBuffer(o.l1ToL2MessagesTreeRoot),
    blocks_tree_root: toBuffer(o.blocksTreeRoot),
    private_kernel_vk_tree_root: toBuffer(o.privateKernelVkTreeRoot),
    public_data_tree_root: toBuffer(o.publicDataTreeRoot),
    global_variables_hash: toBuffer(o.globalVariablesHash),
  };
}

interface MsgpackContractDeploymentData {
  deployer_public_key: MsgpackPoint;
  constructor_vk_hash: Buffer;
  function_tree_root: Buffer;
  contract_address_salt: Buffer;
  portal_contract_address: Buffer;
}

export function toContractDeploymentData(o: MsgpackContractDeploymentData): ContractDeploymentData {
  if (o.deployer_public_key === undefined) {
    throw new Error('Expected deployer_public_key in ContractDeploymentData deserialization');
  }
  if (o.constructor_vk_hash === undefined) {
    throw new Error('Expected constructor_vk_hash in ContractDeploymentData deserialization');
  }
  if (o.function_tree_root === undefined) {
    throw new Error('Expected function_tree_root in ContractDeploymentData deserialization');
  }
  if (o.contract_address_salt === undefined) {
    throw new Error('Expected contract_address_salt in ContractDeploymentData deserialization');
  }
  if (o.portal_contract_address === undefined) {
    throw new Error('Expected portal_contract_address in ContractDeploymentData deserialization');
  }
  return new ContractDeploymentData(
    toPoint(o.deployer_public_key),
    Fr.fromBuffer(o.constructor_vk_hash),
    Fr.fromBuffer(o.function_tree_root),
    Fr.fromBuffer(o.contract_address_salt),
    Address.fromBuffer(o.portal_contract_address),
  );
}

export function fromContractDeploymentData(o: ContractDeploymentData): MsgpackContractDeploymentData {
  if (o.deployerPublicKey === undefined) {
    throw new Error('Expected deployerPublicKey in ContractDeploymentData serialization');
  }
  if (o.constructorVkHash === undefined) {
    throw new Error('Expected constructorVkHash in ContractDeploymentData serialization');
  }
  if (o.functionTreeRoot === undefined) {
    throw new Error('Expected functionTreeRoot in ContractDeploymentData serialization');
  }
  if (o.contractAddressSalt === undefined) {
    throw new Error('Expected contractAddressSalt in ContractDeploymentData serialization');
  }
  if (o.portalContractAddress === undefined) {
    throw new Error('Expected portalContractAddress in ContractDeploymentData serialization');
  }
  return {
    deployer_public_key: fromPoint(o.deployerPublicKey),
    constructor_vk_hash: toBuffer(o.constructorVkHash),
    function_tree_root: toBuffer(o.functionTreeRoot),
    contract_address_salt: toBuffer(o.contractAddressSalt),
    portal_contract_address: toBuffer(o.portalContractAddress),
  };
}

interface MsgpackTxContext {
  is_fee_payment_tx: boolean;
  is_rebate_payment_tx: boolean;
  is_contract_deployment_tx: boolean;
  contract_deployment_data: MsgpackContractDeploymentData;
  chain_id: Buffer;
  version: Buffer;
}

export function toTxContext(o: MsgpackTxContext): TxContext {
  if (o.is_fee_payment_tx === undefined) {
    throw new Error('Expected is_fee_payment_tx in TxContext deserialization');
  }
  if (o.is_rebate_payment_tx === undefined) {
    throw new Error('Expected is_rebate_payment_tx in TxContext deserialization');
  }
  if (o.is_contract_deployment_tx === undefined) {
    throw new Error('Expected is_contract_deployment_tx in TxContext deserialization');
  }
  if (o.contract_deployment_data === undefined) {
    throw new Error('Expected contract_deployment_data in TxContext deserialization');
  }
  if (o.chain_id === undefined) {
    throw new Error('Expected chain_id in TxContext deserialization');
  }
  if (o.version === undefined) {
    throw new Error('Expected version in TxContext deserialization');
  }
  return new TxContext(
    o.is_fee_payment_tx,
    o.is_rebate_payment_tx,
    o.is_contract_deployment_tx,
    toContractDeploymentData(o.contract_deployment_data),
    Fr.fromBuffer(o.chain_id),
    Fr.fromBuffer(o.version),
  );
}

export function fromTxContext(o: TxContext): MsgpackTxContext {
  if (o.isFeePaymentTx === undefined) {
    throw new Error('Expected isFeePaymentTx in TxContext serialization');
  }
  if (o.isRebatePaymentTx === undefined) {
    throw new Error('Expected isRebatePaymentTx in TxContext serialization');
  }
  if (o.isContractDeploymentTx === undefined) {
    throw new Error('Expected isContractDeploymentTx in TxContext serialization');
  }
  if (o.contractDeploymentData === undefined) {
    throw new Error('Expected contractDeploymentData in TxContext serialization');
  }
  if (o.chainId === undefined) {
    throw new Error('Expected chainId in TxContext serialization');
  }
  if (o.version === undefined) {
    throw new Error('Expected version in TxContext serialization');
  }
  return {
    is_fee_payment_tx: o.isFeePaymentTx,
    is_rebate_payment_tx: o.isRebatePaymentTx,
    is_contract_deployment_tx: o.isContractDeploymentTx,
    contract_deployment_data: fromContractDeploymentData(o.contractDeploymentData),
    chain_id: toBuffer(o.chainId),
    version: toBuffer(o.version),
  };
}

interface MsgpackCombinedConstantData {
  block_data: MsgpackHistoricBlockData;
  tx_context: MsgpackTxContext;
}

export function toCombinedConstantData(o: MsgpackCombinedConstantData): CombinedConstantData {
  if (o.block_data === undefined) {
    throw new Error('Expected block_data in CombinedConstantData deserialization');
  }
  if (o.tx_context === undefined) {
    throw new Error('Expected tx_context in CombinedConstantData deserialization');
  }
  return new CombinedConstantData(toHistoricBlockData(o.block_data), toTxContext(o.tx_context));
}

export function fromCombinedConstantData(o: CombinedConstantData): MsgpackCombinedConstantData {
  if (o.blockData === undefined) {
    throw new Error('Expected blockData in CombinedConstantData serialization');
  }
  if (o.txContext === undefined) {
    throw new Error('Expected txContext in CombinedConstantData serialization');
  }
  return {
    block_data: fromHistoricBlockData(o.blockData),
    tx_context: fromTxContext(o.txContext),
  };
}

interface MsgpackKernelCircuitPublicInputs {
  end: MsgpackCombinedAccumulatedData;
  constants: MsgpackCombinedConstantData;
  is_private: boolean;
}

export function toKernelCircuitPublicInputs(o: MsgpackKernelCircuitPublicInputs): KernelCircuitPublicInputs {
  if (o.end === undefined) {
    throw new Error('Expected end in KernelCircuitPublicInputs deserialization');
  }
  if (o.constants === undefined) {
    throw new Error('Expected constants in KernelCircuitPublicInputs deserialization');
  }
  if (o.is_private === undefined) {
    throw new Error('Expected is_private in KernelCircuitPublicInputs deserialization');
  }
  return new KernelCircuitPublicInputs(
    toCombinedAccumulatedData(o.end),
    toCombinedConstantData(o.constants),
    o.is_private,
  );
}

export function fromKernelCircuitPublicInputs(o: KernelCircuitPublicInputs): MsgpackKernelCircuitPublicInputs {
  if (o.end === undefined) {
    throw new Error('Expected end in KernelCircuitPublicInputs serialization');
  }
  if (o.constants === undefined) {
    throw new Error('Expected constants in KernelCircuitPublicInputs serialization');
  }
  if (o.isPrivate === undefined) {
    throw new Error('Expected isPrivate in KernelCircuitPublicInputs serialization');
  }
  return {
    end: fromCombinedAccumulatedData(o.end),
    constants: fromCombinedConstantData(o.constants),
    is_private: o.isPrivate,
  };
}

interface MsgpackVerificationKeyData {
  circuit_type: number;
  circuit_size: number;
  num_public_inputs: number;
  commitments: Record<string, MsgpackG1AffineElement>;
  contains_recursive_proof: boolean;
  recursive_proof_public_input_indices: number[];
}

export function toVerificationKeyData(o: MsgpackVerificationKeyData): VerificationKeyData {
  if (o.circuit_type === undefined) {
    throw new Error('Expected circuit_type in VerificationKeyData deserialization');
  }
  if (o.circuit_size === undefined) {
    throw new Error('Expected circuit_size in VerificationKeyData deserialization');
  }
  if (o.num_public_inputs === undefined) {
    throw new Error('Expected num_public_inputs in VerificationKeyData deserialization');
  }
  if (o.commitments === undefined) {
    throw new Error('Expected commitments in VerificationKeyData deserialization');
  }
  if (o.contains_recursive_proof === undefined) {
    throw new Error('Expected contains_recursive_proof in VerificationKeyData deserialization');
  }
  if (o.recursive_proof_public_input_indices === undefined) {
    throw new Error('Expected recursive_proof_public_input_indices in VerificationKeyData deserialization');
  }
  return new VerificationKeyData(
    o.circuit_type,
    o.circuit_size,
    o.num_public_inputs,
    mapValues(o.commitments, (v: MsgpackG1AffineElement) => toG1AffineElement(v)),
    o.contains_recursive_proof,
    o.recursive_proof_public_input_indices.map((v: number) => v),
  );
}

export function fromVerificationKeyData(o: VerificationKeyData): MsgpackVerificationKeyData {
  if (o.circuitType === undefined) {
    throw new Error('Expected circuitType in VerificationKeyData serialization');
  }
  if (o.circuitSize === undefined) {
    throw new Error('Expected circuitSize in VerificationKeyData serialization');
  }
  if (o.numPublicInputs === undefined) {
    throw new Error('Expected numPublicInputs in VerificationKeyData serialization');
  }
  if (o.commitments === undefined) {
    throw new Error('Expected commitments in VerificationKeyData serialization');
  }
  if (o.containsRecursiveProof === undefined) {
    throw new Error('Expected containsRecursiveProof in VerificationKeyData serialization');
  }
  if (o.recursiveProofPublicInputIndices === undefined) {
    throw new Error('Expected recursiveProofPublicInputIndices in VerificationKeyData serialization');
  }
  return {
    circuit_type: o.circuitType,
    circuit_size: o.circuitSize,
    num_public_inputs: o.numPublicInputs,
    commitments: mapValues(o.commitments, (v: G1AffineElement) => fromG1AffineElement(v)),
    contains_recursive_proof: o.containsRecursiveProof,
    recursive_proof_public_input_indices: o.recursiveProofPublicInputIndices.map((v: number) => v),
  };
}

interface MsgpackPreviousKernelData {
  public_inputs: MsgpackKernelCircuitPublicInputs;
  proof: Buffer;
  vk: MsgpackVerificationKeyData;
  vk_index: number;
  vk_path: Tuple<Buffer, 3>;
}

export function toPreviousKernelData(o: MsgpackPreviousKernelData): PreviousKernelData {
  if (o.public_inputs === undefined) {
    throw new Error('Expected public_inputs in PreviousKernelData deserialization');
  }
  if (o.proof === undefined) {
    throw new Error('Expected proof in PreviousKernelData deserialization');
  }
  if (o.vk === undefined) {
    throw new Error('Expected vk in PreviousKernelData deserialization');
  }
  if (o.vk_index === undefined) {
    throw new Error('Expected vk_index in PreviousKernelData deserialization');
  }
  if (o.vk_path === undefined) {
    throw new Error('Expected vk_path in PreviousKernelData deserialization');
  }
  return new PreviousKernelData(
    toKernelCircuitPublicInputs(o.public_inputs),
    Proof.fromMsgpackBuffer(o.proof),
    toVerificationKeyData(o.vk),
    o.vk_index,
    mapTuple(o.vk_path, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function fromPreviousKernelData(o: PreviousKernelData): MsgpackPreviousKernelData {
  if (o.publicInputs === undefined) {
    throw new Error('Expected publicInputs in PreviousKernelData serialization');
  }
  if (o.proof === undefined) {
    throw new Error('Expected proof in PreviousKernelData serialization');
  }
  if (o.vk === undefined) {
    throw new Error('Expected vk in PreviousKernelData serialization');
  }
  if (o.vkIndex === undefined) {
    throw new Error('Expected vkIndex in PreviousKernelData serialization');
  }
  if (o.vkPath === undefined) {
    throw new Error('Expected vkPath in PreviousKernelData serialization');
  }
  return {
    public_inputs: fromKernelCircuitPublicInputs(o.publicInputs),
    proof: o.proof.toMsgpackBuffer(),
    vk: fromVerificationKeyData(o.vk),
    vk_index: o.vkIndex,
    vk_path: mapTuple(o.vkPath, (v: Fr) => toBuffer(v)),
  };
}

interface MsgpackTxRequest {
  origin: Buffer;
  function_data: MsgpackFunctionData;
  args_hash: Buffer;
  tx_context: MsgpackTxContext;
}

export function toTxRequest(o: MsgpackTxRequest): TxRequest {
  if (o.origin === undefined) {
    throw new Error('Expected origin in TxRequest deserialization');
  }
  if (o.function_data === undefined) {
    throw new Error('Expected function_data in TxRequest deserialization');
  }
  if (o.args_hash === undefined) {
    throw new Error('Expected args_hash in TxRequest deserialization');
  }
  if (o.tx_context === undefined) {
    throw new Error('Expected tx_context in TxRequest deserialization');
  }
  return new TxRequest(
    Address.fromBuffer(o.origin),
    toFunctionData(o.function_data),
    Fr.fromBuffer(o.args_hash),
    toTxContext(o.tx_context),
  );
}

export function fromTxRequest(o: TxRequest): MsgpackTxRequest {
  if (o.origin === undefined) {
    throw new Error('Expected origin in TxRequest serialization');
  }
  if (o.functionData === undefined) {
    throw new Error('Expected functionData in TxRequest serialization');
  }
  if (o.argsHash === undefined) {
    throw new Error('Expected argsHash in TxRequest serialization');
  }
  if (o.txContext === undefined) {
    throw new Error('Expected txContext in TxRequest serialization');
  }
  return {
    origin: toBuffer(o.origin),
    function_data: fromFunctionData(o.functionData),
    args_hash: toBuffer(o.argsHash),
    tx_context: fromTxContext(o.txContext),
  };
}

interface MsgpackCallContext {
  msg_sender: Buffer;
  storage_contract_address: Buffer;
  portal_contract_address: Buffer;
  function_selector: MsgpackFunctionSelector;
  is_delegate_call: boolean;
  is_static_call: boolean;
  is_contract_deployment: boolean;
}

export function toCallContext(o: MsgpackCallContext): CallContext {
  if (o.msg_sender === undefined) {
    throw new Error('Expected msg_sender in CallContext deserialization');
  }
  if (o.storage_contract_address === undefined) {
    throw new Error('Expected storage_contract_address in CallContext deserialization');
  }
  if (o.portal_contract_address === undefined) {
    throw new Error('Expected portal_contract_address in CallContext deserialization');
  }
  if (o.function_selector === undefined) {
    throw new Error('Expected function_selector in CallContext deserialization');
  }
  if (o.is_delegate_call === undefined) {
    throw new Error('Expected is_delegate_call in CallContext deserialization');
  }
  if (o.is_static_call === undefined) {
    throw new Error('Expected is_static_call in CallContext deserialization');
  }
  if (o.is_contract_deployment === undefined) {
    throw new Error('Expected is_contract_deployment in CallContext deserialization');
  }
  return new CallContext(
    Address.fromBuffer(o.msg_sender),
    Address.fromBuffer(o.storage_contract_address),
    Fr.fromBuffer(o.portal_contract_address),
    toFunctionSelector(o.function_selector),
    o.is_delegate_call,
    o.is_static_call,
    o.is_contract_deployment,
  );
}

export function fromCallContext(o: CallContext): MsgpackCallContext {
  if (o.msgSender === undefined) {
    throw new Error('Expected msgSender in CallContext serialization');
  }
  if (o.storageContractAddress === undefined) {
    throw new Error('Expected storageContractAddress in CallContext serialization');
  }
  if (o.portalContractAddress === undefined) {
    throw new Error('Expected portalContractAddress in CallContext serialization');
  }
  if (o.functionSelector === undefined) {
    throw new Error('Expected functionSelector in CallContext serialization');
  }
  if (o.isDelegateCall === undefined) {
    throw new Error('Expected isDelegateCall in CallContext serialization');
  }
  if (o.isStaticCall === undefined) {
    throw new Error('Expected isStaticCall in CallContext serialization');
  }
  if (o.isContractDeployment === undefined) {
    throw new Error('Expected isContractDeployment in CallContext serialization');
  }
  return {
    msg_sender: toBuffer(o.msgSender),
    storage_contract_address: toBuffer(o.storageContractAddress),
    portal_contract_address: toBuffer(o.portalContractAddress),
    function_selector: fromFunctionSelector(o.functionSelector),
    is_delegate_call: o.isDelegateCall,
    is_static_call: o.isStaticCall,
    is_contract_deployment: o.isContractDeployment,
  };
}

interface MsgpackPrivateCircuitPublicInputs {
  call_context: MsgpackCallContext;
  args_hash: Buffer;
  return_values: Tuple<Buffer, 4>;
  read_requests: Tuple<Buffer, 32>;
  new_commitments: Tuple<Buffer, 16>;
  new_nullifiers: Tuple<Buffer, 16>;
  nullified_commitments: Tuple<Buffer, 16>;
  private_call_stack: Tuple<Buffer, 4>;
  public_call_stack: Tuple<Buffer, 4>;
  new_l2_to_l1_msgs: Tuple<Buffer, 2>;
  encrypted_logs_hash: Tuple<Buffer, 2>;
  unencrypted_logs_hash: Tuple<Buffer, 2>;
  encrypted_log_preimages_length: Buffer;
  unencrypted_log_preimages_length: Buffer;
  historic_block_data: MsgpackHistoricBlockData;
  contract_deployment_data: MsgpackContractDeploymentData;
  chain_id: Buffer;
  version: Buffer;
}

export function toPrivateCircuitPublicInputs(o: MsgpackPrivateCircuitPublicInputs): PrivateCircuitPublicInputs {
  if (o.call_context === undefined) {
    throw new Error('Expected call_context in PrivateCircuitPublicInputs deserialization');
  }
  if (o.args_hash === undefined) {
    throw new Error('Expected args_hash in PrivateCircuitPublicInputs deserialization');
  }
  if (o.return_values === undefined) {
    throw new Error('Expected return_values in PrivateCircuitPublicInputs deserialization');
  }
  if (o.read_requests === undefined) {
    throw new Error('Expected read_requests in PrivateCircuitPublicInputs deserialization');
  }
  if (o.new_commitments === undefined) {
    throw new Error('Expected new_commitments in PrivateCircuitPublicInputs deserialization');
  }
  if (o.new_nullifiers === undefined) {
    throw new Error('Expected new_nullifiers in PrivateCircuitPublicInputs deserialization');
  }
  if (o.nullified_commitments === undefined) {
    throw new Error('Expected nullified_commitments in PrivateCircuitPublicInputs deserialization');
  }
  if (o.private_call_stack === undefined) {
    throw new Error('Expected private_call_stack in PrivateCircuitPublicInputs deserialization');
  }
  if (o.public_call_stack === undefined) {
    throw new Error('Expected public_call_stack in PrivateCircuitPublicInputs deserialization');
  }
  if (o.new_l2_to_l1_msgs === undefined) {
    throw new Error('Expected new_l2_to_l1_msgs in PrivateCircuitPublicInputs deserialization');
  }
  if (o.encrypted_logs_hash === undefined) {
    throw new Error('Expected encrypted_logs_hash in PrivateCircuitPublicInputs deserialization');
  }
  if (o.unencrypted_logs_hash === undefined) {
    throw new Error('Expected unencrypted_logs_hash in PrivateCircuitPublicInputs deserialization');
  }
  if (o.encrypted_log_preimages_length === undefined) {
    throw new Error('Expected encrypted_log_preimages_length in PrivateCircuitPublicInputs deserialization');
  }
  if (o.unencrypted_log_preimages_length === undefined) {
    throw new Error('Expected unencrypted_log_preimages_length in PrivateCircuitPublicInputs deserialization');
  }
  if (o.historic_block_data === undefined) {
    throw new Error('Expected historic_block_data in PrivateCircuitPublicInputs deserialization');
  }
  if (o.contract_deployment_data === undefined) {
    throw new Error('Expected contract_deployment_data in PrivateCircuitPublicInputs deserialization');
  }
  if (o.chain_id === undefined) {
    throw new Error('Expected chain_id in PrivateCircuitPublicInputs deserialization');
  }
  if (o.version === undefined) {
    throw new Error('Expected version in PrivateCircuitPublicInputs deserialization');
  }
  return new PrivateCircuitPublicInputs(
    toCallContext(o.call_context),
    Fr.fromBuffer(o.args_hash),
    mapTuple(o.return_values, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.read_requests, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_commitments, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_nullifiers, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.nullified_commitments, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.private_call_stack, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.public_call_stack, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_l2_to_l1_msgs, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.encrypted_logs_hash, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.unencrypted_logs_hash, (v: Buffer) => Fr.fromBuffer(v)),
    Fr.fromBuffer(o.encrypted_log_preimages_length),
    Fr.fromBuffer(o.unencrypted_log_preimages_length),
    toHistoricBlockData(o.historic_block_data),
    toContractDeploymentData(o.contract_deployment_data),
    Fr.fromBuffer(o.chain_id),
    Fr.fromBuffer(o.version),
  );
}

export function fromPrivateCircuitPublicInputs(o: PrivateCircuitPublicInputs): MsgpackPrivateCircuitPublicInputs {
  if (o.callContext === undefined) {
    throw new Error('Expected callContext in PrivateCircuitPublicInputs serialization');
  }
  if (o.argsHash === undefined) {
    throw new Error('Expected argsHash in PrivateCircuitPublicInputs serialization');
  }
  if (o.returnValues === undefined) {
    throw new Error('Expected returnValues in PrivateCircuitPublicInputs serialization');
  }
  if (o.readRequests === undefined) {
    throw new Error('Expected readRequests in PrivateCircuitPublicInputs serialization');
  }
  if (o.newCommitments === undefined) {
    throw new Error('Expected newCommitments in PrivateCircuitPublicInputs serialization');
  }
  if (o.newNullifiers === undefined) {
    throw new Error('Expected newNullifiers in PrivateCircuitPublicInputs serialization');
  }
  if (o.nullifiedCommitments === undefined) {
    throw new Error('Expected nullifiedCommitments in PrivateCircuitPublicInputs serialization');
  }
  if (o.privateCallStack === undefined) {
    throw new Error('Expected privateCallStack in PrivateCircuitPublicInputs serialization');
  }
  if (o.publicCallStack === undefined) {
    throw new Error('Expected publicCallStack in PrivateCircuitPublicInputs serialization');
  }
  if (o.newL2ToL1Msgs === undefined) {
    throw new Error('Expected newL2ToL1Msgs in PrivateCircuitPublicInputs serialization');
  }
  if (o.encryptedLogsHash === undefined) {
    throw new Error('Expected encryptedLogsHash in PrivateCircuitPublicInputs serialization');
  }
  if (o.unencryptedLogsHash === undefined) {
    throw new Error('Expected unencryptedLogsHash in PrivateCircuitPublicInputs serialization');
  }
  if (o.encryptedLogPreimagesLength === undefined) {
    throw new Error('Expected encryptedLogPreimagesLength in PrivateCircuitPublicInputs serialization');
  }
  if (o.unencryptedLogPreimagesLength === undefined) {
    throw new Error('Expected unencryptedLogPreimagesLength in PrivateCircuitPublicInputs serialization');
  }
  if (o.historicBlockData === undefined) {
    throw new Error('Expected historicBlockData in PrivateCircuitPublicInputs serialization');
  }
  if (o.contractDeploymentData === undefined) {
    throw new Error('Expected contractDeploymentData in PrivateCircuitPublicInputs serialization');
  }
  if (o.chainId === undefined) {
    throw new Error('Expected chainId in PrivateCircuitPublicInputs serialization');
  }
  if (o.version === undefined) {
    throw new Error('Expected version in PrivateCircuitPublicInputs serialization');
  }
  return {
    call_context: fromCallContext(o.callContext),
    args_hash: toBuffer(o.argsHash),
    return_values: mapTuple(o.returnValues, (v: Fr) => toBuffer(v)),
    read_requests: mapTuple(o.readRequests, (v: Fr) => toBuffer(v)),
    new_commitments: mapTuple(o.newCommitments, (v: Fr) => toBuffer(v)),
    new_nullifiers: mapTuple(o.newNullifiers, (v: Fr) => toBuffer(v)),
    nullified_commitments: mapTuple(o.nullifiedCommitments, (v: Fr) => toBuffer(v)),
    private_call_stack: mapTuple(o.privateCallStack, (v: Fr) => toBuffer(v)),
    public_call_stack: mapTuple(o.publicCallStack, (v: Fr) => toBuffer(v)),
    new_l2_to_l1_msgs: mapTuple(o.newL2ToL1Msgs, (v: Fr) => toBuffer(v)),
    encrypted_logs_hash: mapTuple(o.encryptedLogsHash, (v: Fr) => toBuffer(v)),
    unencrypted_logs_hash: mapTuple(o.unencryptedLogsHash, (v: Fr) => toBuffer(v)),
    encrypted_log_preimages_length: toBuffer(o.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: toBuffer(o.unencryptedLogPreimagesLength),
    historic_block_data: fromHistoricBlockData(o.historicBlockData),
    contract_deployment_data: fromContractDeploymentData(o.contractDeploymentData),
    chain_id: toBuffer(o.chainId),
    version: toBuffer(o.version),
  };
}

interface MsgpackPrivateCallStackItem {
  contract_address: Buffer;
  function_data: MsgpackFunctionData;
  public_inputs: MsgpackPrivateCircuitPublicInputs;
  is_execution_request: boolean;
}

export function toPrivateCallStackItem(o: MsgpackPrivateCallStackItem): PrivateCallStackItem {
  if (o.contract_address === undefined) {
    throw new Error('Expected contract_address in PrivateCallStackItem deserialization');
  }
  if (o.function_data === undefined) {
    throw new Error('Expected function_data in PrivateCallStackItem deserialization');
  }
  if (o.public_inputs === undefined) {
    throw new Error('Expected public_inputs in PrivateCallStackItem deserialization');
  }
  if (o.is_execution_request === undefined) {
    throw new Error('Expected is_execution_request in PrivateCallStackItem deserialization');
  }
  return new PrivateCallStackItem(
    Address.fromBuffer(o.contract_address),
    toFunctionData(o.function_data),
    toPrivateCircuitPublicInputs(o.public_inputs),
    o.is_execution_request,
  );
}

export function fromPrivateCallStackItem(o: PrivateCallStackItem): MsgpackPrivateCallStackItem {
  if (o.contractAddress === undefined) {
    throw new Error('Expected contractAddress in PrivateCallStackItem serialization');
  }
  if (o.functionData === undefined) {
    throw new Error('Expected functionData in PrivateCallStackItem serialization');
  }
  if (o.publicInputs === undefined) {
    throw new Error('Expected publicInputs in PrivateCallStackItem serialization');
  }
  if (o.isExecutionRequest === undefined) {
    throw new Error('Expected isExecutionRequest in PrivateCallStackItem serialization');
  }
  return {
    contract_address: toBuffer(o.contractAddress),
    function_data: fromFunctionData(o.functionData),
    public_inputs: fromPrivateCircuitPublicInputs(o.publicInputs),
    is_execution_request: o.isExecutionRequest,
  };
}

interface MsgpackMembershipWitness4 {
  leaf_index: Buffer;
  sibling_path: Tuple<Buffer, 4>;
}

export function toMembershipWitness4(o: MsgpackMembershipWitness4): MembershipWitness4 {
  if (o.leaf_index === undefined) {
    throw new Error('Expected leaf_index in MembershipWitness4 deserialization');
  }
  if (o.sibling_path === undefined) {
    throw new Error('Expected sibling_path in MembershipWitness4 deserialization');
  }
  return new MembershipWitness4(
    Fr.fromBuffer(o.leaf_index),
    mapTuple(o.sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function fromMembershipWitness4(o: MembershipWitness4): MsgpackMembershipWitness4 {
  if (o.leafIndex === undefined) {
    throw new Error('Expected leafIndex in MembershipWitness4 serialization');
  }
  if (o.siblingPath === undefined) {
    throw new Error('Expected siblingPath in MembershipWitness4 serialization');
  }
  return {
    leaf_index: toBuffer(o.leafIndex),
    sibling_path: mapTuple(o.siblingPath, (v: Fr) => toBuffer(v)),
  };
}

interface MsgpackMembershipWitness16 {
  leaf_index: Buffer;
  sibling_path: Tuple<Buffer, 16>;
}

export function toMembershipWitness16(o: MsgpackMembershipWitness16): MembershipWitness16 {
  if (o.leaf_index === undefined) {
    throw new Error('Expected leaf_index in MembershipWitness16 deserialization');
  }
  if (o.sibling_path === undefined) {
    throw new Error('Expected sibling_path in MembershipWitness16 deserialization');
  }
  return new MembershipWitness16(
    Fr.fromBuffer(o.leaf_index),
    mapTuple(o.sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function fromMembershipWitness16(o: MembershipWitness16): MsgpackMembershipWitness16 {
  if (o.leafIndex === undefined) {
    throw new Error('Expected leafIndex in MembershipWitness16 serialization');
  }
  if (o.siblingPath === undefined) {
    throw new Error('Expected siblingPath in MembershipWitness16 serialization');
  }
  return {
    leaf_index: toBuffer(o.leafIndex),
    sibling_path: mapTuple(o.siblingPath, (v: Fr) => toBuffer(v)),
  };
}

interface MsgpackReadRequestMembershipWitness {
  leaf_index: Buffer;
  sibling_path: Tuple<Buffer, 32>;
  is_transient: boolean;
  hint_to_commitment: Buffer;
}

export function toReadRequestMembershipWitness(o: MsgpackReadRequestMembershipWitness): ReadRequestMembershipWitness {
  if (o.leaf_index === undefined) {
    throw new Error('Expected leaf_index in ReadRequestMembershipWitness deserialization');
  }
  if (o.sibling_path === undefined) {
    throw new Error('Expected sibling_path in ReadRequestMembershipWitness deserialization');
  }
  if (o.is_transient === undefined) {
    throw new Error('Expected is_transient in ReadRequestMembershipWitness deserialization');
  }
  if (o.hint_to_commitment === undefined) {
    throw new Error('Expected hint_to_commitment in ReadRequestMembershipWitness deserialization');
  }
  return new ReadRequestMembershipWitness(
    Fr.fromBuffer(o.leaf_index),
    mapTuple(o.sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
    o.is_transient,
    Fr.fromBuffer(o.hint_to_commitment),
  );
}

export function fromReadRequestMembershipWitness(o: ReadRequestMembershipWitness): MsgpackReadRequestMembershipWitness {
  if (o.leafIndex === undefined) {
    throw new Error('Expected leafIndex in ReadRequestMembershipWitness serialization');
  }
  if (o.siblingPath === undefined) {
    throw new Error('Expected siblingPath in ReadRequestMembershipWitness serialization');
  }
  if (o.isTransient === undefined) {
    throw new Error('Expected isTransient in ReadRequestMembershipWitness serialization');
  }
  if (o.hintToCommitment === undefined) {
    throw new Error('Expected hintToCommitment in ReadRequestMembershipWitness serialization');
  }
  return {
    leaf_index: toBuffer(o.leafIndex),
    sibling_path: mapTuple(o.siblingPath, (v: Fr) => toBuffer(v)),
    is_transient: o.isTransient,
    hint_to_commitment: toBuffer(o.hintToCommitment),
  };
}

interface MsgpackPrivateCallData {
  call_stack_item: MsgpackPrivateCallStackItem;
  private_call_stack_preimages: Tuple<MsgpackPrivateCallStackItem, 4>;
  proof: Buffer;
  vk: MsgpackVerificationKeyData;
  function_leaf_membership_witness: MsgpackMembershipWitness4;
  contract_leaf_membership_witness: MsgpackMembershipWitness16;
  read_request_membership_witnesses: Tuple<MsgpackReadRequestMembershipWitness, 32>;
  portal_contract_address: Buffer;
  acir_hash: Buffer;
}

export function toPrivateCallData(o: MsgpackPrivateCallData): PrivateCallData {
  if (o.call_stack_item === undefined) {
    throw new Error('Expected call_stack_item in PrivateCallData deserialization');
  }
  if (o.private_call_stack_preimages === undefined) {
    throw new Error('Expected private_call_stack_preimages in PrivateCallData deserialization');
  }
  if (o.proof === undefined) {
    throw new Error('Expected proof in PrivateCallData deserialization');
  }
  if (o.vk === undefined) {
    throw new Error('Expected vk in PrivateCallData deserialization');
  }
  if (o.function_leaf_membership_witness === undefined) {
    throw new Error('Expected function_leaf_membership_witness in PrivateCallData deserialization');
  }
  if (o.contract_leaf_membership_witness === undefined) {
    throw new Error('Expected contract_leaf_membership_witness in PrivateCallData deserialization');
  }
  if (o.read_request_membership_witnesses === undefined) {
    throw new Error('Expected read_request_membership_witnesses in PrivateCallData deserialization');
  }
  if (o.portal_contract_address === undefined) {
    throw new Error('Expected portal_contract_address in PrivateCallData deserialization');
  }
  if (o.acir_hash === undefined) {
    throw new Error('Expected acir_hash in PrivateCallData deserialization');
  }
  return new PrivateCallData(
    toPrivateCallStackItem(o.call_stack_item),
    mapTuple(o.private_call_stack_preimages, (v: MsgpackPrivateCallStackItem) => toPrivateCallStackItem(v)),
    Proof.fromMsgpackBuffer(o.proof),
    toVerificationKeyData(o.vk),
    toMembershipWitness4(o.function_leaf_membership_witness),
    toMembershipWitness16(o.contract_leaf_membership_witness),
    mapTuple(o.read_request_membership_witnesses, (v: MsgpackReadRequestMembershipWitness) =>
      toReadRequestMembershipWitness(v),
    ),
    Fr.fromBuffer(o.portal_contract_address),
    Fr.fromBuffer(o.acir_hash),
  );
}

export function fromPrivateCallData(o: PrivateCallData): MsgpackPrivateCallData {
  if (o.callStackItem === undefined) {
    throw new Error('Expected callStackItem in PrivateCallData serialization');
  }
  if (o.privateCallStackPreimages === undefined) {
    throw new Error('Expected privateCallStackPreimages in PrivateCallData serialization');
  }
  if (o.proof === undefined) {
    throw new Error('Expected proof in PrivateCallData serialization');
  }
  if (o.vk === undefined) {
    throw new Error('Expected vk in PrivateCallData serialization');
  }
  if (o.functionLeafMembershipWitness === undefined) {
    throw new Error('Expected functionLeafMembershipWitness in PrivateCallData serialization');
  }
  if (o.contractLeafMembershipWitness === undefined) {
    throw new Error('Expected contractLeafMembershipWitness in PrivateCallData serialization');
  }
  if (o.readRequestMembershipWitnesses === undefined) {
    throw new Error('Expected readRequestMembershipWitnesses in PrivateCallData serialization');
  }
  if (o.portalContractAddress === undefined) {
    throw new Error('Expected portalContractAddress in PrivateCallData serialization');
  }
  if (o.acirHash === undefined) {
    throw new Error('Expected acirHash in PrivateCallData serialization');
  }
  return {
    call_stack_item: fromPrivateCallStackItem(o.callStackItem),
    private_call_stack_preimages: mapTuple(o.privateCallStackPreimages, (v: PrivateCallStackItem) =>
      fromPrivateCallStackItem(v),
    ),
    proof: o.proof.toMsgpackBuffer(),
    vk: fromVerificationKeyData(o.vk),
    function_leaf_membership_witness: fromMembershipWitness4(o.functionLeafMembershipWitness),
    contract_leaf_membership_witness: fromMembershipWitness16(o.contractLeafMembershipWitness),
    read_request_membership_witnesses: mapTuple(o.readRequestMembershipWitnesses, (v: ReadRequestMembershipWitness) =>
      fromReadRequestMembershipWitness(v),
    ),
    portal_contract_address: toBuffer(o.portalContractAddress),
    acir_hash: toBuffer(o.acirHash),
  };
}

interface MsgpackPrivateKernelInputsInit {
  tx_request: MsgpackTxRequest;
  private_call: MsgpackPrivateCallData;
}

export function toPrivateKernelInputsInit(o: MsgpackPrivateKernelInputsInit): PrivateKernelInputsInit {
  if (o.tx_request === undefined) {
    throw new Error('Expected tx_request in PrivateKernelInputsInit deserialization');
  }
  if (o.private_call === undefined) {
    throw new Error('Expected private_call in PrivateKernelInputsInit deserialization');
  }
  return new PrivateKernelInputsInit(toTxRequest(o.tx_request), toPrivateCallData(o.private_call));
}

export function fromPrivateKernelInputsInit(o: PrivateKernelInputsInit): MsgpackPrivateKernelInputsInit {
  if (o.txRequest === undefined) {
    throw new Error('Expected txRequest in PrivateKernelInputsInit serialization');
  }
  if (o.privateCall === undefined) {
    throw new Error('Expected privateCall in PrivateKernelInputsInit serialization');
  }
  return {
    tx_request: fromTxRequest(o.txRequest),
    private_call: fromPrivateCallData(o.privateCall),
  };
}

interface MsgpackCircuitError {
  code: number;
  message: string;
}

export function toCircuitError(o: MsgpackCircuitError): CircuitError {
  if (o.code === undefined) {
    throw new Error('Expected code in CircuitError deserialization');
  }
  if (o.message === undefined) {
    throw new Error('Expected message in CircuitError deserialization');
  }
  return new CircuitError(o.code, o.message);
}

export function fromCircuitError(o: CircuitError): MsgpackCircuitError {
  if (o.code === undefined) {
    throw new Error('Expected code in CircuitError serialization');
  }
  if (o.message === undefined) {
    throw new Error('Expected message in CircuitError serialization');
  }
  return {
    code: o.code,
    message: o.message,
  };
}

interface MsgpackPrivateKernelInputsInner {
  previous_kernel: MsgpackPreviousKernelData;
  private_call: MsgpackPrivateCallData;
}

export function toPrivateKernelInputsInner(o: MsgpackPrivateKernelInputsInner): PrivateKernelInputsInner {
  if (o.previous_kernel === undefined) {
    throw new Error('Expected previous_kernel in PrivateKernelInputsInner deserialization');
  }
  if (o.private_call === undefined) {
    throw new Error('Expected private_call in PrivateKernelInputsInner deserialization');
  }
  return new PrivateKernelInputsInner(toPreviousKernelData(o.previous_kernel), toPrivateCallData(o.private_call));
}

export function fromPrivateKernelInputsInner(o: PrivateKernelInputsInner): MsgpackPrivateKernelInputsInner {
  if (o.previousKernel === undefined) {
    throw new Error('Expected previousKernel in PrivateKernelInputsInner serialization');
  }
  if (o.privateCall === undefined) {
    throw new Error('Expected privateCall in PrivateKernelInputsInner serialization');
  }
  return {
    previous_kernel: fromPreviousKernelData(o.previousKernel),
    private_call: fromPrivateCallData(o.privateCall),
  };
}

interface MsgpackPrivateKernelInputsOrdering {
  previous_kernel: MsgpackPreviousKernelData;
  read_commitment_hints: Tuple<Buffer, 128>;
  nullifier_commitment_hints: Tuple<Buffer, 64>;
}

export function toPrivateKernelInputsOrdering(o: MsgpackPrivateKernelInputsOrdering): PrivateKernelInputsOrdering {
  if (o.previous_kernel === undefined) {
    throw new Error('Expected previous_kernel in PrivateKernelInputsOrdering deserialization');
  }
  if (o.read_commitment_hints === undefined) {
    throw new Error('Expected read_commitment_hints in PrivateKernelInputsOrdering deserialization');
  }
  if (o.nullifier_commitment_hints === undefined) {
    throw new Error('Expected nullifier_commitment_hints in PrivateKernelInputsOrdering deserialization');
  }
  return new PrivateKernelInputsOrdering(
    toPreviousKernelData(o.previous_kernel),
    mapTuple(o.read_commitment_hints, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.nullifier_commitment_hints, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function fromPrivateKernelInputsOrdering(o: PrivateKernelInputsOrdering): MsgpackPrivateKernelInputsOrdering {
  if (o.previousKernel === undefined) {
    throw new Error('Expected previousKernel in PrivateKernelInputsOrdering serialization');
  }
  if (o.readCommitmentHints === undefined) {
    throw new Error('Expected readCommitmentHints in PrivateKernelInputsOrdering serialization');
  }
  if (o.nullifierCommitmentHints === undefined) {
    throw new Error('Expected nullifierCommitmentHints in PrivateKernelInputsOrdering serialization');
  }
  return {
    previous_kernel: fromPreviousKernelData(o.previousKernel),
    read_commitment_hints: mapTuple(o.readCommitmentHints, (v: Fr) => toBuffer(v)),
    nullifier_commitment_hints: mapTuple(o.nullifierCommitmentHints, (v: Fr) => toBuffer(v)),
  };
}

interface MsgpackFinalAccumulatedData {
  aggregation_object: MsgpackNativeAggregationState;
  new_commitments: Tuple<Buffer, 64>;
  new_nullifiers: Tuple<Buffer, 64>;
  nullified_commitments: Tuple<Buffer, 64>;
  private_call_stack: Tuple<Buffer, 8>;
  public_call_stack: Tuple<Buffer, 8>;
  new_l2_to_l1_msgs: Tuple<Buffer, 2>;
  encrypted_logs_hash: Tuple<Buffer, 2>;
  unencrypted_logs_hash: Tuple<Buffer, 2>;
  encrypted_log_preimages_length: Buffer;
  unencrypted_log_preimages_length: Buffer;
  new_contracts: Tuple<MsgpackNewContractData, 1>;
  optionally_revealed_data: Tuple<MsgpackOptionallyRevealedData, 4>;
}

export function toFinalAccumulatedData(o: MsgpackFinalAccumulatedData): FinalAccumulatedData {
  if (o.aggregation_object === undefined) {
    throw new Error('Expected aggregation_object in FinalAccumulatedData deserialization');
  }
  if (o.new_commitments === undefined) {
    throw new Error('Expected new_commitments in FinalAccumulatedData deserialization');
  }
  if (o.new_nullifiers === undefined) {
    throw new Error('Expected new_nullifiers in FinalAccumulatedData deserialization');
  }
  if (o.nullified_commitments === undefined) {
    throw new Error('Expected nullified_commitments in FinalAccumulatedData deserialization');
  }
  if (o.private_call_stack === undefined) {
    throw new Error('Expected private_call_stack in FinalAccumulatedData deserialization');
  }
  if (o.public_call_stack === undefined) {
    throw new Error('Expected public_call_stack in FinalAccumulatedData deserialization');
  }
  if (o.new_l2_to_l1_msgs === undefined) {
    throw new Error('Expected new_l2_to_l1_msgs in FinalAccumulatedData deserialization');
  }
  if (o.encrypted_logs_hash === undefined) {
    throw new Error('Expected encrypted_logs_hash in FinalAccumulatedData deserialization');
  }
  if (o.unencrypted_logs_hash === undefined) {
    throw new Error('Expected unencrypted_logs_hash in FinalAccumulatedData deserialization');
  }
  if (o.encrypted_log_preimages_length === undefined) {
    throw new Error('Expected encrypted_log_preimages_length in FinalAccumulatedData deserialization');
  }
  if (o.unencrypted_log_preimages_length === undefined) {
    throw new Error('Expected unencrypted_log_preimages_length in FinalAccumulatedData deserialization');
  }
  if (o.new_contracts === undefined) {
    throw new Error('Expected new_contracts in FinalAccumulatedData deserialization');
  }
  if (o.optionally_revealed_data === undefined) {
    throw new Error('Expected optionally_revealed_data in FinalAccumulatedData deserialization');
  }
  return new FinalAccumulatedData(
    toNativeAggregationState(o.aggregation_object),
    mapTuple(o.new_commitments, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_nullifiers, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.nullified_commitments, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.private_call_stack, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.public_call_stack, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_l2_to_l1_msgs, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.encrypted_logs_hash, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.unencrypted_logs_hash, (v: Buffer) => Fr.fromBuffer(v)),
    Fr.fromBuffer(o.encrypted_log_preimages_length),
    Fr.fromBuffer(o.unencrypted_log_preimages_length),
    mapTuple(o.new_contracts, (v: MsgpackNewContractData) => toNewContractData(v)),
    mapTuple(o.optionally_revealed_data, (v: MsgpackOptionallyRevealedData) => toOptionallyRevealedData(v)),
  );
}

export function fromFinalAccumulatedData(o: FinalAccumulatedData): MsgpackFinalAccumulatedData {
  if (o.aggregationObject === undefined) {
    throw new Error('Expected aggregationObject in FinalAccumulatedData serialization');
  }
  if (o.newCommitments === undefined) {
    throw new Error('Expected newCommitments in FinalAccumulatedData serialization');
  }
  if (o.newNullifiers === undefined) {
    throw new Error('Expected newNullifiers in FinalAccumulatedData serialization');
  }
  if (o.nullifiedCommitments === undefined) {
    throw new Error('Expected nullifiedCommitments in FinalAccumulatedData serialization');
  }
  if (o.privateCallStack === undefined) {
    throw new Error('Expected privateCallStack in FinalAccumulatedData serialization');
  }
  if (o.publicCallStack === undefined) {
    throw new Error('Expected publicCallStack in FinalAccumulatedData serialization');
  }
  if (o.newL2ToL1Msgs === undefined) {
    throw new Error('Expected newL2ToL1Msgs in FinalAccumulatedData serialization');
  }
  if (o.encryptedLogsHash === undefined) {
    throw new Error('Expected encryptedLogsHash in FinalAccumulatedData serialization');
  }
  if (o.unencryptedLogsHash === undefined) {
    throw new Error('Expected unencryptedLogsHash in FinalAccumulatedData serialization');
  }
  if (o.encryptedLogPreimagesLength === undefined) {
    throw new Error('Expected encryptedLogPreimagesLength in FinalAccumulatedData serialization');
  }
  if (o.unencryptedLogPreimagesLength === undefined) {
    throw new Error('Expected unencryptedLogPreimagesLength in FinalAccumulatedData serialization');
  }
  if (o.newContracts === undefined) {
    throw new Error('Expected newContracts in FinalAccumulatedData serialization');
  }
  if (o.optionallyRevealedData === undefined) {
    throw new Error('Expected optionallyRevealedData in FinalAccumulatedData serialization');
  }
  return {
    aggregation_object: fromNativeAggregationState(o.aggregationObject),
    new_commitments: mapTuple(o.newCommitments, (v: Fr) => toBuffer(v)),
    new_nullifiers: mapTuple(o.newNullifiers, (v: Fr) => toBuffer(v)),
    nullified_commitments: mapTuple(o.nullifiedCommitments, (v: Fr) => toBuffer(v)),
    private_call_stack: mapTuple(o.privateCallStack, (v: Fr) => toBuffer(v)),
    public_call_stack: mapTuple(o.publicCallStack, (v: Fr) => toBuffer(v)),
    new_l2_to_l1_msgs: mapTuple(o.newL2ToL1Msgs, (v: Fr) => toBuffer(v)),
    encrypted_logs_hash: mapTuple(o.encryptedLogsHash, (v: Fr) => toBuffer(v)),
    unencrypted_logs_hash: mapTuple(o.unencryptedLogsHash, (v: Fr) => toBuffer(v)),
    encrypted_log_preimages_length: toBuffer(o.encryptedLogPreimagesLength),
    unencrypted_log_preimages_length: toBuffer(o.unencryptedLogPreimagesLength),
    new_contracts: mapTuple(o.newContracts, (v: NewContractData) => fromNewContractData(v)),
    optionally_revealed_data: mapTuple(o.optionallyRevealedData, (v: OptionallyRevealedData) =>
      fromOptionallyRevealedData(v),
    ),
  };
}

interface MsgpackKernelCircuitPublicInputsFinal {
  end: MsgpackFinalAccumulatedData;
  constants: MsgpackCombinedConstantData;
  is_private: boolean;
}

export function toKernelCircuitPublicInputsFinal(
  o: MsgpackKernelCircuitPublicInputsFinal,
): KernelCircuitPublicInputsFinal {
  if (o.end === undefined) {
    throw new Error('Expected end in KernelCircuitPublicInputsFinal deserialization');
  }
  if (o.constants === undefined) {
    throw new Error('Expected constants in KernelCircuitPublicInputsFinal deserialization');
  }
  if (o.is_private === undefined) {
    throw new Error('Expected is_private in KernelCircuitPublicInputsFinal deserialization');
  }
  return new KernelCircuitPublicInputsFinal(
    toFinalAccumulatedData(o.end),
    toCombinedConstantData(o.constants),
    o.is_private,
  );
}

export function fromKernelCircuitPublicInputsFinal(
  o: KernelCircuitPublicInputsFinal,
): MsgpackKernelCircuitPublicInputsFinal {
  if (o.end === undefined) {
    throw new Error('Expected end in KernelCircuitPublicInputsFinal serialization');
  }
  if (o.constants === undefined) {
    throw new Error('Expected constants in KernelCircuitPublicInputsFinal serialization');
  }
  if (o.isPrivate === undefined) {
    throw new Error('Expected isPrivate in KernelCircuitPublicInputsFinal serialization');
  }
  return {
    end: fromFinalAccumulatedData(o.end),
    constants: fromCombinedConstantData(o.constants),
    is_private: o.isPrivate,
  };
}

interface MsgpackContractStorageUpdateRequest {
  storage_slot: Buffer;
  old_value: Buffer;
  new_value: Buffer;
}

export function toContractStorageUpdateRequest(o: MsgpackContractStorageUpdateRequest): ContractStorageUpdateRequest {
  if (o.storage_slot === undefined) {
    throw new Error('Expected storage_slot in ContractStorageUpdateRequest deserialization');
  }
  if (o.old_value === undefined) {
    throw new Error('Expected old_value in ContractStorageUpdateRequest deserialization');
  }
  if (o.new_value === undefined) {
    throw new Error('Expected new_value in ContractStorageUpdateRequest deserialization');
  }
  return new ContractStorageUpdateRequest(
    Fr.fromBuffer(o.storage_slot),
    Fr.fromBuffer(o.old_value),
    Fr.fromBuffer(o.new_value),
  );
}

export function fromContractStorageUpdateRequest(o: ContractStorageUpdateRequest): MsgpackContractStorageUpdateRequest {
  if (o.storageSlot === undefined) {
    throw new Error('Expected storageSlot in ContractStorageUpdateRequest serialization');
  }
  if (o.oldValue === undefined) {
    throw new Error('Expected oldValue in ContractStorageUpdateRequest serialization');
  }
  if (o.newValue === undefined) {
    throw new Error('Expected newValue in ContractStorageUpdateRequest serialization');
  }
  return {
    storage_slot: toBuffer(o.storageSlot),
    old_value: toBuffer(o.oldValue),
    new_value: toBuffer(o.newValue),
  };
}

interface MsgpackContractStorageRead {
  storage_slot: Buffer;
  current_value: Buffer;
}

export function toContractStorageRead(o: MsgpackContractStorageRead): ContractStorageRead {
  if (o.storage_slot === undefined) {
    throw new Error('Expected storage_slot in ContractStorageRead deserialization');
  }
  if (o.current_value === undefined) {
    throw new Error('Expected current_value in ContractStorageRead deserialization');
  }
  return new ContractStorageRead(Fr.fromBuffer(o.storage_slot), Fr.fromBuffer(o.current_value));
}

export function fromContractStorageRead(o: ContractStorageRead): MsgpackContractStorageRead {
  if (o.storageSlot === undefined) {
    throw new Error('Expected storageSlot in ContractStorageRead serialization');
  }
  if (o.currentValue === undefined) {
    throw new Error('Expected currentValue in ContractStorageRead serialization');
  }
  return {
    storage_slot: toBuffer(o.storageSlot),
    current_value: toBuffer(o.currentValue),
  };
}

interface MsgpackPublicCircuitPublicInputs {
  call_context: MsgpackCallContext;
  args_hash: Buffer;
  return_values: Tuple<Buffer, 4>;
  contract_storage_update_requests: Tuple<MsgpackContractStorageUpdateRequest, 16>;
  contract_storage_reads: Tuple<MsgpackContractStorageRead, 16>;
  public_call_stack: Tuple<Buffer, 4>;
  new_commitments: Tuple<Buffer, 16>;
  new_nullifiers: Tuple<Buffer, 16>;
  new_l2_to_l1_msgs: Tuple<Buffer, 2>;
  unencrypted_logs_hash: Tuple<Buffer, 2>;
  unencrypted_log_preimages_length: Buffer;
  historic_block_data: MsgpackHistoricBlockData;
  prover_address: Buffer;
}

export function toPublicCircuitPublicInputs(o: MsgpackPublicCircuitPublicInputs): PublicCircuitPublicInputs {
  if (o.call_context === undefined) {
    throw new Error('Expected call_context in PublicCircuitPublicInputs deserialization');
  }
  if (o.args_hash === undefined) {
    throw new Error('Expected args_hash in PublicCircuitPublicInputs deserialization');
  }
  if (o.return_values === undefined) {
    throw new Error('Expected return_values in PublicCircuitPublicInputs deserialization');
  }
  if (o.contract_storage_update_requests === undefined) {
    throw new Error('Expected contract_storage_update_requests in PublicCircuitPublicInputs deserialization');
  }
  if (o.contract_storage_reads === undefined) {
    throw new Error('Expected contract_storage_reads in PublicCircuitPublicInputs deserialization');
  }
  if (o.public_call_stack === undefined) {
    throw new Error('Expected public_call_stack in PublicCircuitPublicInputs deserialization');
  }
  if (o.new_commitments === undefined) {
    throw new Error('Expected new_commitments in PublicCircuitPublicInputs deserialization');
  }
  if (o.new_nullifiers === undefined) {
    throw new Error('Expected new_nullifiers in PublicCircuitPublicInputs deserialization');
  }
  if (o.new_l2_to_l1_msgs === undefined) {
    throw new Error('Expected new_l2_to_l1_msgs in PublicCircuitPublicInputs deserialization');
  }
  if (o.unencrypted_logs_hash === undefined) {
    throw new Error('Expected unencrypted_logs_hash in PublicCircuitPublicInputs deserialization');
  }
  if (o.unencrypted_log_preimages_length === undefined) {
    throw new Error('Expected unencrypted_log_preimages_length in PublicCircuitPublicInputs deserialization');
  }
  if (o.historic_block_data === undefined) {
    throw new Error('Expected historic_block_data in PublicCircuitPublicInputs deserialization');
  }
  if (o.prover_address === undefined) {
    throw new Error('Expected prover_address in PublicCircuitPublicInputs deserialization');
  }
  return new PublicCircuitPublicInputs(
    toCallContext(o.call_context),
    Fr.fromBuffer(o.args_hash),
    mapTuple(o.return_values, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.contract_storage_update_requests, (v: MsgpackContractStorageUpdateRequest) =>
      toContractStorageUpdateRequest(v),
    ),
    mapTuple(o.contract_storage_reads, (v: MsgpackContractStorageRead) => toContractStorageRead(v)),
    mapTuple(o.public_call_stack, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_commitments, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_nullifiers, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_l2_to_l1_msgs, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.unencrypted_logs_hash, (v: Buffer) => Fr.fromBuffer(v)),
    Fr.fromBuffer(o.unencrypted_log_preimages_length),
    toHistoricBlockData(o.historic_block_data),
    Address.fromBuffer(o.prover_address),
  );
}

export function fromPublicCircuitPublicInputs(o: PublicCircuitPublicInputs): MsgpackPublicCircuitPublicInputs {
  if (o.callContext === undefined) {
    throw new Error('Expected callContext in PublicCircuitPublicInputs serialization');
  }
  if (o.argsHash === undefined) {
    throw new Error('Expected argsHash in PublicCircuitPublicInputs serialization');
  }
  if (o.returnValues === undefined) {
    throw new Error('Expected returnValues in PublicCircuitPublicInputs serialization');
  }
  if (o.contractStorageUpdateRequests === undefined) {
    throw new Error('Expected contractStorageUpdateRequests in PublicCircuitPublicInputs serialization');
  }
  if (o.contractStorageReads === undefined) {
    throw new Error('Expected contractStorageReads in PublicCircuitPublicInputs serialization');
  }
  if (o.publicCallStack === undefined) {
    throw new Error('Expected publicCallStack in PublicCircuitPublicInputs serialization');
  }
  if (o.newCommitments === undefined) {
    throw new Error('Expected newCommitments in PublicCircuitPublicInputs serialization');
  }
  if (o.newNullifiers === undefined) {
    throw new Error('Expected newNullifiers in PublicCircuitPublicInputs serialization');
  }
  if (o.newL2ToL1Msgs === undefined) {
    throw new Error('Expected newL2ToL1Msgs in PublicCircuitPublicInputs serialization');
  }
  if (o.unencryptedLogsHash === undefined) {
    throw new Error('Expected unencryptedLogsHash in PublicCircuitPublicInputs serialization');
  }
  if (o.unencryptedLogPreimagesLength === undefined) {
    throw new Error('Expected unencryptedLogPreimagesLength in PublicCircuitPublicInputs serialization');
  }
  if (o.historicBlockData === undefined) {
    throw new Error('Expected historicBlockData in PublicCircuitPublicInputs serialization');
  }
  if (o.proverAddress === undefined) {
    throw new Error('Expected proverAddress in PublicCircuitPublicInputs serialization');
  }
  return {
    call_context: fromCallContext(o.callContext),
    args_hash: toBuffer(o.argsHash),
    return_values: mapTuple(o.returnValues, (v: Fr) => toBuffer(v)),
    contract_storage_update_requests: mapTuple(o.contractStorageUpdateRequests, (v: ContractStorageUpdateRequest) =>
      fromContractStorageUpdateRequest(v),
    ),
    contract_storage_reads: mapTuple(o.contractStorageReads, (v: ContractStorageRead) => fromContractStorageRead(v)),
    public_call_stack: mapTuple(o.publicCallStack, (v: Fr) => toBuffer(v)),
    new_commitments: mapTuple(o.newCommitments, (v: Fr) => toBuffer(v)),
    new_nullifiers: mapTuple(o.newNullifiers, (v: Fr) => toBuffer(v)),
    new_l2_to_l1_msgs: mapTuple(o.newL2ToL1Msgs, (v: Fr) => toBuffer(v)),
    unencrypted_logs_hash: mapTuple(o.unencryptedLogsHash, (v: Fr) => toBuffer(v)),
    unencrypted_log_preimages_length: toBuffer(o.unencryptedLogPreimagesLength),
    historic_block_data: fromHistoricBlockData(o.historicBlockData),
    prover_address: toBuffer(o.proverAddress),
  };
}

interface MsgpackPublicCallStackItem {
  contract_address: Buffer;
  function_data: MsgpackFunctionData;
  public_inputs: MsgpackPublicCircuitPublicInputs;
  is_execution_request: boolean;
}

export function toPublicCallStackItem(o: MsgpackPublicCallStackItem): PublicCallStackItem {
  if (o.contract_address === undefined) {
    throw new Error('Expected contract_address in PublicCallStackItem deserialization');
  }
  if (o.function_data === undefined) {
    throw new Error('Expected function_data in PublicCallStackItem deserialization');
  }
  if (o.public_inputs === undefined) {
    throw new Error('Expected public_inputs in PublicCallStackItem deserialization');
  }
  if (o.is_execution_request === undefined) {
    throw new Error('Expected is_execution_request in PublicCallStackItem deserialization');
  }
  return new PublicCallStackItem(
    Address.fromBuffer(o.contract_address),
    toFunctionData(o.function_data),
    toPublicCircuitPublicInputs(o.public_inputs),
    o.is_execution_request,
  );
}

export function fromPublicCallStackItem(o: PublicCallStackItem): MsgpackPublicCallStackItem {
  if (o.contractAddress === undefined) {
    throw new Error('Expected contractAddress in PublicCallStackItem serialization');
  }
  if (o.functionData === undefined) {
    throw new Error('Expected functionData in PublicCallStackItem serialization');
  }
  if (o.publicInputs === undefined) {
    throw new Error('Expected publicInputs in PublicCallStackItem serialization');
  }
  if (o.isExecutionRequest === undefined) {
    throw new Error('Expected isExecutionRequest in PublicCallStackItem serialization');
  }
  return {
    contract_address: toBuffer(o.contractAddress),
    function_data: fromFunctionData(o.functionData),
    public_inputs: fromPublicCircuitPublicInputs(o.publicInputs),
    is_execution_request: o.isExecutionRequest,
  };
}

interface MsgpackPublicCallData {
  call_stack_item: MsgpackPublicCallStackItem;
  public_call_stack_preimages: Tuple<MsgpackPublicCallStackItem, 4>;
  proof: Buffer;
  portal_contract_address: Buffer;
  bytecode_hash: Buffer;
}

export function toPublicCallData(o: MsgpackPublicCallData): PublicCallData {
  if (o.call_stack_item === undefined) {
    throw new Error('Expected call_stack_item in PublicCallData deserialization');
  }
  if (o.public_call_stack_preimages === undefined) {
    throw new Error('Expected public_call_stack_preimages in PublicCallData deserialization');
  }
  if (o.proof === undefined) {
    throw new Error('Expected proof in PublicCallData deserialization');
  }
  if (o.portal_contract_address === undefined) {
    throw new Error('Expected portal_contract_address in PublicCallData deserialization');
  }
  if (o.bytecode_hash === undefined) {
    throw new Error('Expected bytecode_hash in PublicCallData deserialization');
  }
  return new PublicCallData(
    toPublicCallStackItem(o.call_stack_item),
    mapTuple(o.public_call_stack_preimages, (v: MsgpackPublicCallStackItem) => toPublicCallStackItem(v)),
    Proof.fromMsgpackBuffer(o.proof),
    Fr.fromBuffer(o.portal_contract_address),
    Fr.fromBuffer(o.bytecode_hash),
  );
}

export function fromPublicCallData(o: PublicCallData): MsgpackPublicCallData {
  if (o.callStackItem === undefined) {
    throw new Error('Expected callStackItem in PublicCallData serialization');
  }
  if (o.publicCallStackPreimages === undefined) {
    throw new Error('Expected publicCallStackPreimages in PublicCallData serialization');
  }
  if (o.proof === undefined) {
    throw new Error('Expected proof in PublicCallData serialization');
  }
  if (o.portalContractAddress === undefined) {
    throw new Error('Expected portalContractAddress in PublicCallData serialization');
  }
  if (o.bytecodeHash === undefined) {
    throw new Error('Expected bytecodeHash in PublicCallData serialization');
  }
  return {
    call_stack_item: fromPublicCallStackItem(o.callStackItem),
    public_call_stack_preimages: mapTuple(o.publicCallStackPreimages, (v: PublicCallStackItem) =>
      fromPublicCallStackItem(v),
    ),
    proof: o.proof.toMsgpackBuffer(),
    portal_contract_address: toBuffer(o.portalContractAddress),
    bytecode_hash: toBuffer(o.bytecodeHash),
  };
}

interface MsgpackPublicKernelInputs {
  previous_kernel: MsgpackPreviousKernelData;
  public_call: MsgpackPublicCallData;
}

export function toPublicKernelInputs(o: MsgpackPublicKernelInputs): PublicKernelInputs {
  if (o.previous_kernel === undefined) {
    throw new Error('Expected previous_kernel in PublicKernelInputs deserialization');
  }
  if (o.public_call === undefined) {
    throw new Error('Expected public_call in PublicKernelInputs deserialization');
  }
  return new PublicKernelInputs(toPreviousKernelData(o.previous_kernel), toPublicCallData(o.public_call));
}

export function fromPublicKernelInputs(o: PublicKernelInputs): MsgpackPublicKernelInputs {
  if (o.previousKernel === undefined) {
    throw new Error('Expected previousKernel in PublicKernelInputs serialization');
  }
  if (o.publicCall === undefined) {
    throw new Error('Expected publicCall in PublicKernelInputs serialization');
  }
  return {
    previous_kernel: fromPreviousKernelData(o.previousKernel),
    public_call: fromPublicCallData(o.publicCall),
  };
}

interface MsgpackAppendOnlyTreeSnapshot {
  root: Buffer;
  next_available_leaf_index: number;
}

export function toAppendOnlyTreeSnapshot(o: MsgpackAppendOnlyTreeSnapshot): AppendOnlyTreeSnapshot {
  if (o.root === undefined) {
    throw new Error('Expected root in AppendOnlyTreeSnapshot deserialization');
  }
  if (o.next_available_leaf_index === undefined) {
    throw new Error('Expected next_available_leaf_index in AppendOnlyTreeSnapshot deserialization');
  }
  return new AppendOnlyTreeSnapshot(Fr.fromBuffer(o.root), o.next_available_leaf_index);
}

export function fromAppendOnlyTreeSnapshot(o: AppendOnlyTreeSnapshot): MsgpackAppendOnlyTreeSnapshot {
  if (o.root === undefined) {
    throw new Error('Expected root in AppendOnlyTreeSnapshot serialization');
  }
  if (o.nextAvailableLeafIndex === undefined) {
    throw new Error('Expected nextAvailableLeafIndex in AppendOnlyTreeSnapshot serialization');
  }
  return {
    root: toBuffer(o.root),
    next_available_leaf_index: o.nextAvailableLeafIndex,
  };
}

interface MsgpackNullifierLeafPreimage {
  leaf_value: Buffer;
  next_value: Buffer;
  next_index: number;
}

export function toNullifierLeafPreimage(o: MsgpackNullifierLeafPreimage): NullifierLeafPreimage {
  if (o.leaf_value === undefined) {
    throw new Error('Expected leaf_value in NullifierLeafPreimage deserialization');
  }
  if (o.next_value === undefined) {
    throw new Error('Expected next_value in NullifierLeafPreimage deserialization');
  }
  if (o.next_index === undefined) {
    throw new Error('Expected next_index in NullifierLeafPreimage deserialization');
  }
  return new NullifierLeafPreimage(Fr.fromBuffer(o.leaf_value), Fr.fromBuffer(o.next_value), o.next_index);
}

export function fromNullifierLeafPreimage(o: NullifierLeafPreimage): MsgpackNullifierLeafPreimage {
  if (o.leafValue === undefined) {
    throw new Error('Expected leafValue in NullifierLeafPreimage serialization');
  }
  if (o.nextValue === undefined) {
    throw new Error('Expected nextValue in NullifierLeafPreimage serialization');
  }
  if (o.nextIndex === undefined) {
    throw new Error('Expected nextIndex in NullifierLeafPreimage serialization');
  }
  return {
    leaf_value: toBuffer(o.leafValue),
    next_value: toBuffer(o.nextValue),
    next_index: o.nextIndex,
  };
}

interface MsgpackConstantRollupData {
  start_historic_blocks_tree_roots_snapshot: MsgpackAppendOnlyTreeSnapshot;
  private_kernel_vk_tree_root: Buffer;
  public_kernel_vk_tree_root: Buffer;
  base_rollup_vk_hash: Buffer;
  merge_rollup_vk_hash: Buffer;
  global_variables: MsgpackGlobalVariables;
}

export function toConstantRollupData(o: MsgpackConstantRollupData): ConstantRollupData {
  if (o.start_historic_blocks_tree_roots_snapshot === undefined) {
    throw new Error('Expected start_historic_blocks_tree_roots_snapshot in ConstantRollupData deserialization');
  }
  if (o.private_kernel_vk_tree_root === undefined) {
    throw new Error('Expected private_kernel_vk_tree_root in ConstantRollupData deserialization');
  }
  if (o.public_kernel_vk_tree_root === undefined) {
    throw new Error('Expected public_kernel_vk_tree_root in ConstantRollupData deserialization');
  }
  if (o.base_rollup_vk_hash === undefined) {
    throw new Error('Expected base_rollup_vk_hash in ConstantRollupData deserialization');
  }
  if (o.merge_rollup_vk_hash === undefined) {
    throw new Error('Expected merge_rollup_vk_hash in ConstantRollupData deserialization');
  }
  if (o.global_variables === undefined) {
    throw new Error('Expected global_variables in ConstantRollupData deserialization');
  }
  return new ConstantRollupData(
    toAppendOnlyTreeSnapshot(o.start_historic_blocks_tree_roots_snapshot),
    Fr.fromBuffer(o.private_kernel_vk_tree_root),
    Fr.fromBuffer(o.public_kernel_vk_tree_root),
    Fr.fromBuffer(o.base_rollup_vk_hash),
    Fr.fromBuffer(o.merge_rollup_vk_hash),
    toGlobalVariables(o.global_variables),
  );
}

export function fromConstantRollupData(o: ConstantRollupData): MsgpackConstantRollupData {
  if (o.startHistoricBlocksTreeRootsSnapshot === undefined) {
    throw new Error('Expected startHistoricBlocksTreeRootsSnapshot in ConstantRollupData serialization');
  }
  if (o.privateKernelVkTreeRoot === undefined) {
    throw new Error('Expected privateKernelVkTreeRoot in ConstantRollupData serialization');
  }
  if (o.publicKernelVkTreeRoot === undefined) {
    throw new Error('Expected publicKernelVkTreeRoot in ConstantRollupData serialization');
  }
  if (o.baseRollupVkHash === undefined) {
    throw new Error('Expected baseRollupVkHash in ConstantRollupData serialization');
  }
  if (o.mergeRollupVkHash === undefined) {
    throw new Error('Expected mergeRollupVkHash in ConstantRollupData serialization');
  }
  if (o.globalVariables === undefined) {
    throw new Error('Expected globalVariables in ConstantRollupData serialization');
  }
  return {
    start_historic_blocks_tree_roots_snapshot: fromAppendOnlyTreeSnapshot(o.startHistoricBlocksTreeRootsSnapshot),
    private_kernel_vk_tree_root: toBuffer(o.privateKernelVkTreeRoot),
    public_kernel_vk_tree_root: toBuffer(o.publicKernelVkTreeRoot),
    base_rollup_vk_hash: toBuffer(o.baseRollupVkHash),
    merge_rollup_vk_hash: toBuffer(o.mergeRollupVkHash),
    global_variables: fromGlobalVariables(o.globalVariables),
  };
}

interface MsgpackBaseRollupInputs {
  kernel_data: Tuple<MsgpackPreviousKernelData, 2>;
  start_private_data_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_nullifier_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_contract_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_public_data_tree_root: Buffer;
  start_historic_blocks_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  low_nullifier_leaf_preimages: Tuple<MsgpackNullifierLeafPreimage, 128>;
  low_nullifier_membership_witness: Tuple<MsgpackMembershipWitness16, 128>;
  new_commitments_subtree_sibling_path: Tuple<Buffer, 25>;
  new_nullifiers_subtree_sibling_path: Tuple<Buffer, 9>;
  new_contracts_subtree_sibling_path: Tuple<Buffer, 15>;
  new_public_data_update_requests_sibling_paths: Tuple<Tuple<Buffer, 254>, 32>;
  new_public_data_reads_sibling_paths: Tuple<Tuple<Buffer, 254>, 32>;
  historic_blocks_tree_root_membership_witnesses: Tuple<MsgpackMembershipWitness16, 2>;
  constants: MsgpackConstantRollupData;
}

export function toBaseRollupInputs(o: MsgpackBaseRollupInputs): BaseRollupInputs {
  if (o.kernel_data === undefined) {
    throw new Error('Expected kernel_data in BaseRollupInputs deserialization');
  }
  if (o.start_private_data_tree_snapshot === undefined) {
    throw new Error('Expected start_private_data_tree_snapshot in BaseRollupInputs deserialization');
  }
  if (o.start_nullifier_tree_snapshot === undefined) {
    throw new Error('Expected start_nullifier_tree_snapshot in BaseRollupInputs deserialization');
  }
  if (o.start_contract_tree_snapshot === undefined) {
    throw new Error('Expected start_contract_tree_snapshot in BaseRollupInputs deserialization');
  }
  if (o.start_public_data_tree_root === undefined) {
    throw new Error('Expected start_public_data_tree_root in BaseRollupInputs deserialization');
  }
  if (o.start_historic_blocks_tree_snapshot === undefined) {
    throw new Error('Expected start_historic_blocks_tree_snapshot in BaseRollupInputs deserialization');
  }
  if (o.low_nullifier_leaf_preimages === undefined) {
    throw new Error('Expected low_nullifier_leaf_preimages in BaseRollupInputs deserialization');
  }
  if (o.low_nullifier_membership_witness === undefined) {
    throw new Error('Expected low_nullifier_membership_witness in BaseRollupInputs deserialization');
  }
  if (o.new_commitments_subtree_sibling_path === undefined) {
    throw new Error('Expected new_commitments_subtree_sibling_path in BaseRollupInputs deserialization');
  }
  if (o.new_nullifiers_subtree_sibling_path === undefined) {
    throw new Error('Expected new_nullifiers_subtree_sibling_path in BaseRollupInputs deserialization');
  }
  if (o.new_contracts_subtree_sibling_path === undefined) {
    throw new Error('Expected new_contracts_subtree_sibling_path in BaseRollupInputs deserialization');
  }
  if (o.new_public_data_update_requests_sibling_paths === undefined) {
    throw new Error('Expected new_public_data_update_requests_sibling_paths in BaseRollupInputs deserialization');
  }
  if (o.new_public_data_reads_sibling_paths === undefined) {
    throw new Error('Expected new_public_data_reads_sibling_paths in BaseRollupInputs deserialization');
  }
  if (o.historic_blocks_tree_root_membership_witnesses === undefined) {
    throw new Error('Expected historic_blocks_tree_root_membership_witnesses in BaseRollupInputs deserialization');
  }
  if (o.constants === undefined) {
    throw new Error('Expected constants in BaseRollupInputs deserialization');
  }
  return new BaseRollupInputs(
    mapTuple(o.kernel_data, (v: MsgpackPreviousKernelData) => toPreviousKernelData(v)),
    toAppendOnlyTreeSnapshot(o.start_private_data_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_nullifier_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_contract_tree_snapshot),
    Fr.fromBuffer(o.start_public_data_tree_root),
    toAppendOnlyTreeSnapshot(o.start_historic_blocks_tree_snapshot),
    mapTuple(o.low_nullifier_leaf_preimages, (v: MsgpackNullifierLeafPreimage) => toNullifierLeafPreimage(v)),
    mapTuple(o.low_nullifier_membership_witness, (v: MsgpackMembershipWitness16) => toMembershipWitness16(v)),
    mapTuple(o.new_commitments_subtree_sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_nullifiers_subtree_sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_contracts_subtree_sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_public_data_update_requests_sibling_paths, (v: Tuple<Buffer, 254>) =>
      mapTuple(v, (v: Buffer) => Fr.fromBuffer(v)),
    ),
    mapTuple(o.new_public_data_reads_sibling_paths, (v: Tuple<Buffer, 254>) =>
      mapTuple(v, (v: Buffer) => Fr.fromBuffer(v)),
    ),
    mapTuple(o.historic_blocks_tree_root_membership_witnesses, (v: MsgpackMembershipWitness16) =>
      toMembershipWitness16(v),
    ),
    toConstantRollupData(o.constants),
  );
}

export function fromBaseRollupInputs(o: BaseRollupInputs): MsgpackBaseRollupInputs {
  if (o.kernelData === undefined) {
    throw new Error('Expected kernelData in BaseRollupInputs serialization');
  }
  if (o.startPrivateDataTreeSnapshot === undefined) {
    throw new Error('Expected startPrivateDataTreeSnapshot in BaseRollupInputs serialization');
  }
  if (o.startNullifierTreeSnapshot === undefined) {
    throw new Error('Expected startNullifierTreeSnapshot in BaseRollupInputs serialization');
  }
  if (o.startContractTreeSnapshot === undefined) {
    throw new Error('Expected startContractTreeSnapshot in BaseRollupInputs serialization');
  }
  if (o.startPublicDataTreeRoot === undefined) {
    throw new Error('Expected startPublicDataTreeRoot in BaseRollupInputs serialization');
  }
  if (o.startHistoricBlocksTreeSnapshot === undefined) {
    throw new Error('Expected startHistoricBlocksTreeSnapshot in BaseRollupInputs serialization');
  }
  if (o.lowNullifierLeafPreimages === undefined) {
    throw new Error('Expected lowNullifierLeafPreimages in BaseRollupInputs serialization');
  }
  if (o.lowNullifierMembershipWitness === undefined) {
    throw new Error('Expected lowNullifierMembershipWitness in BaseRollupInputs serialization');
  }
  if (o.newCommitmentsSubtreeSiblingPath === undefined) {
    throw new Error('Expected newCommitmentsSubtreeSiblingPath in BaseRollupInputs serialization');
  }
  if (o.newNullifiersSubtreeSiblingPath === undefined) {
    throw new Error('Expected newNullifiersSubtreeSiblingPath in BaseRollupInputs serialization');
  }
  if (o.newContractsSubtreeSiblingPath === undefined) {
    throw new Error('Expected newContractsSubtreeSiblingPath in BaseRollupInputs serialization');
  }
  if (o.newPublicDataUpdateRequestsSiblingPaths === undefined) {
    throw new Error('Expected newPublicDataUpdateRequestsSiblingPaths in BaseRollupInputs serialization');
  }
  if (o.newPublicDataReadsSiblingPaths === undefined) {
    throw new Error('Expected newPublicDataReadsSiblingPaths in BaseRollupInputs serialization');
  }
  if (o.historicBlocksTreeRootMembershipWitnesses === undefined) {
    throw new Error('Expected historicBlocksTreeRootMembershipWitnesses in BaseRollupInputs serialization');
  }
  if (o.constants === undefined) {
    throw new Error('Expected constants in BaseRollupInputs serialization');
  }
  return {
    kernel_data: mapTuple(o.kernelData, (v: PreviousKernelData) => fromPreviousKernelData(v)),
    start_private_data_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startPrivateDataTreeSnapshot),
    start_nullifier_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startNullifierTreeSnapshot),
    start_contract_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startContractTreeSnapshot),
    start_public_data_tree_root: toBuffer(o.startPublicDataTreeRoot),
    start_historic_blocks_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startHistoricBlocksTreeSnapshot),
    low_nullifier_leaf_preimages: mapTuple(o.lowNullifierLeafPreimages, (v: NullifierLeafPreimage) =>
      fromNullifierLeafPreimage(v),
    ),
    low_nullifier_membership_witness: mapTuple(o.lowNullifierMembershipWitness, (v: MembershipWitness16) =>
      fromMembershipWitness16(v),
    ),
    new_commitments_subtree_sibling_path: mapTuple(o.newCommitmentsSubtreeSiblingPath, (v: Fr) => toBuffer(v)),
    new_nullifiers_subtree_sibling_path: mapTuple(o.newNullifiersSubtreeSiblingPath, (v: Fr) => toBuffer(v)),
    new_contracts_subtree_sibling_path: mapTuple(o.newContractsSubtreeSiblingPath, (v: Fr) => toBuffer(v)),
    new_public_data_update_requests_sibling_paths: mapTuple(
      o.newPublicDataUpdateRequestsSiblingPaths,
      (v: Tuple<Fr, 254>) => mapTuple(v, (v: Fr) => toBuffer(v)),
    ),
    new_public_data_reads_sibling_paths: mapTuple(o.newPublicDataReadsSiblingPaths, (v: Tuple<Fr, 254>) =>
      mapTuple(v, (v: Fr) => toBuffer(v)),
    ),
    historic_blocks_tree_root_membership_witnesses: mapTuple(
      o.historicBlocksTreeRootMembershipWitnesses,
      (v: MembershipWitness16) => fromMembershipWitness16(v),
    ),
    constants: fromConstantRollupData(o.constants),
  };
}

interface MsgpackBaseOrMergeRollupPublicInputs {
  rollup_type: number;
  rollup_subtree_height: Buffer;
  end_aggregation_object: MsgpackNativeAggregationState;
  constants: MsgpackConstantRollupData;
  start_private_data_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_private_data_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_nullifier_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_nullifier_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_contract_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_contract_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_public_data_tree_root: Buffer;
  end_public_data_tree_root: Buffer;
  calldata_hash: Tuple<Buffer, 2>;
}

export function toBaseOrMergeRollupPublicInputs(
  o: MsgpackBaseOrMergeRollupPublicInputs,
): BaseOrMergeRollupPublicInputs {
  if (o.rollup_type === undefined) {
    throw new Error('Expected rollup_type in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.rollup_subtree_height === undefined) {
    throw new Error('Expected rollup_subtree_height in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.end_aggregation_object === undefined) {
    throw new Error('Expected end_aggregation_object in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.constants === undefined) {
    throw new Error('Expected constants in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.start_private_data_tree_snapshot === undefined) {
    throw new Error('Expected start_private_data_tree_snapshot in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.end_private_data_tree_snapshot === undefined) {
    throw new Error('Expected end_private_data_tree_snapshot in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.start_nullifier_tree_snapshot === undefined) {
    throw new Error('Expected start_nullifier_tree_snapshot in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.end_nullifier_tree_snapshot === undefined) {
    throw new Error('Expected end_nullifier_tree_snapshot in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.start_contract_tree_snapshot === undefined) {
    throw new Error('Expected start_contract_tree_snapshot in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.end_contract_tree_snapshot === undefined) {
    throw new Error('Expected end_contract_tree_snapshot in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.start_public_data_tree_root === undefined) {
    throw new Error('Expected start_public_data_tree_root in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.end_public_data_tree_root === undefined) {
    throw new Error('Expected end_public_data_tree_root in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.calldata_hash === undefined) {
    throw new Error('Expected calldata_hash in BaseOrMergeRollupPublicInputs deserialization');
  }
  return new BaseOrMergeRollupPublicInputs(
    o.rollup_type,
    Fr.fromBuffer(o.rollup_subtree_height),
    toNativeAggregationState(o.end_aggregation_object),
    toConstantRollupData(o.constants),
    toAppendOnlyTreeSnapshot(o.start_private_data_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_private_data_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_nullifier_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_nullifier_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_contract_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_contract_tree_snapshot),
    Fr.fromBuffer(o.start_public_data_tree_root),
    Fr.fromBuffer(o.end_public_data_tree_root),
    mapTuple(o.calldata_hash, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function fromBaseOrMergeRollupPublicInputs(
  o: BaseOrMergeRollupPublicInputs,
): MsgpackBaseOrMergeRollupPublicInputs {
  if (o.rollupType === undefined) {
    throw new Error('Expected rollupType in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.rollupSubtreeHeight === undefined) {
    throw new Error('Expected rollupSubtreeHeight in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.endAggregationObject === undefined) {
    throw new Error('Expected endAggregationObject in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.constants === undefined) {
    throw new Error('Expected constants in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.startPrivateDataTreeSnapshot === undefined) {
    throw new Error('Expected startPrivateDataTreeSnapshot in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.endPrivateDataTreeSnapshot === undefined) {
    throw new Error('Expected endPrivateDataTreeSnapshot in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.startNullifierTreeSnapshot === undefined) {
    throw new Error('Expected startNullifierTreeSnapshot in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.endNullifierTreeSnapshot === undefined) {
    throw new Error('Expected endNullifierTreeSnapshot in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.startContractTreeSnapshot === undefined) {
    throw new Error('Expected startContractTreeSnapshot in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.endContractTreeSnapshot === undefined) {
    throw new Error('Expected endContractTreeSnapshot in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.startPublicDataTreeRoot === undefined) {
    throw new Error('Expected startPublicDataTreeRoot in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.endPublicDataTreeRoot === undefined) {
    throw new Error('Expected endPublicDataTreeRoot in BaseOrMergeRollupPublicInputs serialization');
  }
  if (o.calldataHash === undefined) {
    throw new Error('Expected calldataHash in BaseOrMergeRollupPublicInputs serialization');
  }
  return {
    rollup_type: o.rollupType,
    rollup_subtree_height: toBuffer(o.rollupSubtreeHeight),
    end_aggregation_object: fromNativeAggregationState(o.endAggregationObject),
    constants: fromConstantRollupData(o.constants),
    start_private_data_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startPrivateDataTreeSnapshot),
    end_private_data_tree_snapshot: fromAppendOnlyTreeSnapshot(o.endPrivateDataTreeSnapshot),
    start_nullifier_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startNullifierTreeSnapshot),
    end_nullifier_tree_snapshot: fromAppendOnlyTreeSnapshot(o.endNullifierTreeSnapshot),
    start_contract_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startContractTreeSnapshot),
    end_contract_tree_snapshot: fromAppendOnlyTreeSnapshot(o.endContractTreeSnapshot),
    start_public_data_tree_root: toBuffer(o.startPublicDataTreeRoot),
    end_public_data_tree_root: toBuffer(o.endPublicDataTreeRoot),
    calldata_hash: mapTuple(o.calldataHash, (v: Fr) => toBuffer(v)),
  };
}

interface MsgpackMembershipWitness8 {
  leaf_index: Buffer;
  sibling_path: Tuple<Buffer, 8>;
}

export function toMembershipWitness8(o: MsgpackMembershipWitness8): MembershipWitness8 {
  if (o.leaf_index === undefined) {
    throw new Error('Expected leaf_index in MembershipWitness8 deserialization');
  }
  if (o.sibling_path === undefined) {
    throw new Error('Expected sibling_path in MembershipWitness8 deserialization');
  }
  return new MembershipWitness8(
    Fr.fromBuffer(o.leaf_index),
    mapTuple(o.sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function fromMembershipWitness8(o: MembershipWitness8): MsgpackMembershipWitness8 {
  if (o.leafIndex === undefined) {
    throw new Error('Expected leafIndex in MembershipWitness8 serialization');
  }
  if (o.siblingPath === undefined) {
    throw new Error('Expected siblingPath in MembershipWitness8 serialization');
  }
  return {
    leaf_index: toBuffer(o.leafIndex),
    sibling_path: mapTuple(o.siblingPath, (v: Fr) => toBuffer(v)),
  };
}

interface MsgpackPreviousRollupData {
  base_or_merge_rollup_public_inputs: MsgpackBaseOrMergeRollupPublicInputs;
  proof: Buffer;
  vk: MsgpackVerificationKeyData;
  vk_index: number;
  vk_sibling_path: MsgpackMembershipWitness8;
}

export function toPreviousRollupData(o: MsgpackPreviousRollupData): PreviousRollupData {
  if (o.base_or_merge_rollup_public_inputs === undefined) {
    throw new Error('Expected base_or_merge_rollup_public_inputs in PreviousRollupData deserialization');
  }
  if (o.proof === undefined) {
    throw new Error('Expected proof in PreviousRollupData deserialization');
  }
  if (o.vk === undefined) {
    throw new Error('Expected vk in PreviousRollupData deserialization');
  }
  if (o.vk_index === undefined) {
    throw new Error('Expected vk_index in PreviousRollupData deserialization');
  }
  if (o.vk_sibling_path === undefined) {
    throw new Error('Expected vk_sibling_path in PreviousRollupData deserialization');
  }
  return new PreviousRollupData(
    toBaseOrMergeRollupPublicInputs(o.base_or_merge_rollup_public_inputs),
    Proof.fromMsgpackBuffer(o.proof),
    toVerificationKeyData(o.vk),
    o.vk_index,
    toMembershipWitness8(o.vk_sibling_path),
  );
}

export function fromPreviousRollupData(o: PreviousRollupData): MsgpackPreviousRollupData {
  if (o.baseOrMergeRollupPublicInputs === undefined) {
    throw new Error('Expected baseOrMergeRollupPublicInputs in PreviousRollupData serialization');
  }
  if (o.proof === undefined) {
    throw new Error('Expected proof in PreviousRollupData serialization');
  }
  if (o.vk === undefined) {
    throw new Error('Expected vk in PreviousRollupData serialization');
  }
  if (o.vkIndex === undefined) {
    throw new Error('Expected vkIndex in PreviousRollupData serialization');
  }
  if (o.vkSiblingPath === undefined) {
    throw new Error('Expected vkSiblingPath in PreviousRollupData serialization');
  }
  return {
    base_or_merge_rollup_public_inputs: fromBaseOrMergeRollupPublicInputs(o.baseOrMergeRollupPublicInputs),
    proof: o.proof.toMsgpackBuffer(),
    vk: fromVerificationKeyData(o.vk),
    vk_index: o.vkIndex,
    vk_sibling_path: fromMembershipWitness8(o.vkSiblingPath),
  };
}

interface MsgpackMergeRollupInputs {
  previous_rollup_data: Tuple<MsgpackPreviousRollupData, 2>;
}

export function toMergeRollupInputs(o: MsgpackMergeRollupInputs): MergeRollupInputs {
  if (o.previous_rollup_data === undefined) {
    throw new Error('Expected previous_rollup_data in MergeRollupInputs deserialization');
  }
  return new MergeRollupInputs(
    mapTuple(o.previous_rollup_data, (v: MsgpackPreviousRollupData) => toPreviousRollupData(v)),
  );
}

export function fromMergeRollupInputs(o: MergeRollupInputs): MsgpackMergeRollupInputs {
  if (o.previousRollupData === undefined) {
    throw new Error('Expected previousRollupData in MergeRollupInputs serialization');
  }
  return {
    previous_rollup_data: mapTuple(o.previousRollupData, (v: PreviousRollupData) => fromPreviousRollupData(v)),
  };
}

interface MsgpackRootRollupInputs {
  previous_rollup_data: Tuple<MsgpackPreviousRollupData, 2>;
  new_l1_to_l2_messages: Tuple<Buffer, 16>;
  new_l1_to_l2_messages_tree_root_sibling_path: Tuple<Buffer, 12>;
  start_l1_to_l2_messages_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_historic_blocks_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  new_historic_blocks_tree_sibling_path: Tuple<Buffer, 16>;
}

export function toRootRollupInputs(o: MsgpackRootRollupInputs): RootRollupInputs {
  if (o.previous_rollup_data === undefined) {
    throw new Error('Expected previous_rollup_data in RootRollupInputs deserialization');
  }
  if (o.new_l1_to_l2_messages === undefined) {
    throw new Error('Expected new_l1_to_l2_messages in RootRollupInputs deserialization');
  }
  if (o.new_l1_to_l2_messages_tree_root_sibling_path === undefined) {
    throw new Error('Expected new_l1_to_l2_messages_tree_root_sibling_path in RootRollupInputs deserialization');
  }
  if (o.start_l1_to_l2_messages_tree_snapshot === undefined) {
    throw new Error('Expected start_l1_to_l2_messages_tree_snapshot in RootRollupInputs deserialization');
  }
  if (o.start_historic_blocks_tree_snapshot === undefined) {
    throw new Error('Expected start_historic_blocks_tree_snapshot in RootRollupInputs deserialization');
  }
  if (o.new_historic_blocks_tree_sibling_path === undefined) {
    throw new Error('Expected new_historic_blocks_tree_sibling_path in RootRollupInputs deserialization');
  }
  return new RootRollupInputs(
    mapTuple(o.previous_rollup_data, (v: MsgpackPreviousRollupData) => toPreviousRollupData(v)),
    mapTuple(o.new_l1_to_l2_messages, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_l1_to_l2_messages_tree_root_sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
    toAppendOnlyTreeSnapshot(o.start_l1_to_l2_messages_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_historic_blocks_tree_snapshot),
    mapTuple(o.new_historic_blocks_tree_sibling_path, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function fromRootRollupInputs(o: RootRollupInputs): MsgpackRootRollupInputs {
  if (o.previousRollupData === undefined) {
    throw new Error('Expected previousRollupData in RootRollupInputs serialization');
  }
  if (o.newL1ToL2Messages === undefined) {
    throw new Error('Expected newL1ToL2Messages in RootRollupInputs serialization');
  }
  if (o.newL1ToL2MessagesTreeRootSiblingPath === undefined) {
    throw new Error('Expected newL1ToL2MessagesTreeRootSiblingPath in RootRollupInputs serialization');
  }
  if (o.startL1ToL2MessagesTreeSnapshot === undefined) {
    throw new Error('Expected startL1ToL2MessagesTreeSnapshot in RootRollupInputs serialization');
  }
  if (o.startHistoricBlocksTreeSnapshot === undefined) {
    throw new Error('Expected startHistoricBlocksTreeSnapshot in RootRollupInputs serialization');
  }
  if (o.newHistoricBlocksTreeSiblingPath === undefined) {
    throw new Error('Expected newHistoricBlocksTreeSiblingPath in RootRollupInputs serialization');
  }
  return {
    previous_rollup_data: mapTuple(o.previousRollupData, (v: PreviousRollupData) => fromPreviousRollupData(v)),
    new_l1_to_l2_messages: mapTuple(o.newL1ToL2Messages, (v: Fr) => toBuffer(v)),
    new_l1_to_l2_messages_tree_root_sibling_path: mapTuple(o.newL1ToL2MessagesTreeRootSiblingPath, (v: Fr) =>
      toBuffer(v),
    ),
    start_l1_to_l2_messages_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startL1ToL2MessagesTreeSnapshot),
    start_historic_blocks_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startHistoricBlocksTreeSnapshot),
    new_historic_blocks_tree_sibling_path: mapTuple(o.newHistoricBlocksTreeSiblingPath, (v: Fr) => toBuffer(v)),
  };
}

interface MsgpackRootRollupPublicInputs {
  end_aggregation_object: MsgpackNativeAggregationState;
  global_variables: MsgpackGlobalVariables;
  start_private_data_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_private_data_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_nullifier_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_nullifier_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_contract_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_contract_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_public_data_tree_root: Buffer;
  end_public_data_tree_root: Buffer;
  start_tree_of_historic_private_data_tree_roots_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_tree_of_historic_private_data_tree_roots_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_tree_of_historic_contract_tree_roots_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_tree_of_historic_contract_tree_roots_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_l1_to_l2_messages_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_l1_to_l2_messages_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_historic_blocks_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_historic_blocks_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  calldata_hash: Tuple<Buffer, 2>;
  l1_to_l2_messages_hash: Tuple<Buffer, 2>;
}

export function toRootRollupPublicInputs(o: MsgpackRootRollupPublicInputs): RootRollupPublicInputs {
  if (o.end_aggregation_object === undefined) {
    throw new Error('Expected end_aggregation_object in RootRollupPublicInputs deserialization');
  }
  if (o.global_variables === undefined) {
    throw new Error('Expected global_variables in RootRollupPublicInputs deserialization');
  }
  if (o.start_private_data_tree_snapshot === undefined) {
    throw new Error('Expected start_private_data_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.end_private_data_tree_snapshot === undefined) {
    throw new Error('Expected end_private_data_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.start_nullifier_tree_snapshot === undefined) {
    throw new Error('Expected start_nullifier_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.end_nullifier_tree_snapshot === undefined) {
    throw new Error('Expected end_nullifier_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.start_contract_tree_snapshot === undefined) {
    throw new Error('Expected start_contract_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.end_contract_tree_snapshot === undefined) {
    throw new Error('Expected end_contract_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.start_public_data_tree_root === undefined) {
    throw new Error('Expected start_public_data_tree_root in RootRollupPublicInputs deserialization');
  }
  if (o.end_public_data_tree_root === undefined) {
    throw new Error('Expected end_public_data_tree_root in RootRollupPublicInputs deserialization');
  }
  if (o.start_tree_of_historic_private_data_tree_roots_snapshot === undefined) {
    throw new Error(
      'Expected start_tree_of_historic_private_data_tree_roots_snapshot in RootRollupPublicInputs deserialization',
    );
  }
  if (o.end_tree_of_historic_private_data_tree_roots_snapshot === undefined) {
    throw new Error(
      'Expected end_tree_of_historic_private_data_tree_roots_snapshot in RootRollupPublicInputs deserialization',
    );
  }
  if (o.start_tree_of_historic_contract_tree_roots_snapshot === undefined) {
    throw new Error(
      'Expected start_tree_of_historic_contract_tree_roots_snapshot in RootRollupPublicInputs deserialization',
    );
  }
  if (o.end_tree_of_historic_contract_tree_roots_snapshot === undefined) {
    throw new Error(
      'Expected end_tree_of_historic_contract_tree_roots_snapshot in RootRollupPublicInputs deserialization',
    );
  }
  if (o.start_l1_to_l2_messages_tree_snapshot === undefined) {
    throw new Error('Expected start_l1_to_l2_messages_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.end_l1_to_l2_messages_tree_snapshot === undefined) {
    throw new Error('Expected end_l1_to_l2_messages_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.start_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot === undefined) {
    throw new Error(
      'Expected start_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot in RootRollupPublicInputs deserialization',
    );
  }
  if (o.end_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot === undefined) {
    throw new Error(
      'Expected end_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot in RootRollupPublicInputs deserialization',
    );
  }
  if (o.start_historic_blocks_tree_snapshot === undefined) {
    throw new Error('Expected start_historic_blocks_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.end_historic_blocks_tree_snapshot === undefined) {
    throw new Error('Expected end_historic_blocks_tree_snapshot in RootRollupPublicInputs deserialization');
  }
  if (o.calldata_hash === undefined) {
    throw new Error('Expected calldata_hash in RootRollupPublicInputs deserialization');
  }
  if (o.l1_to_l2_messages_hash === undefined) {
    throw new Error('Expected l1_to_l2_messages_hash in RootRollupPublicInputs deserialization');
  }
  return new RootRollupPublicInputs(
    toNativeAggregationState(o.end_aggregation_object),
    toGlobalVariables(o.global_variables),
    toAppendOnlyTreeSnapshot(o.start_private_data_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_private_data_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_nullifier_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_nullifier_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_contract_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_contract_tree_snapshot),
    Fr.fromBuffer(o.start_public_data_tree_root),
    Fr.fromBuffer(o.end_public_data_tree_root),
    toAppendOnlyTreeSnapshot(o.start_tree_of_historic_private_data_tree_roots_snapshot),
    toAppendOnlyTreeSnapshot(o.end_tree_of_historic_private_data_tree_roots_snapshot),
    toAppendOnlyTreeSnapshot(o.start_tree_of_historic_contract_tree_roots_snapshot),
    toAppendOnlyTreeSnapshot(o.end_tree_of_historic_contract_tree_roots_snapshot),
    toAppendOnlyTreeSnapshot(o.start_l1_to_l2_messages_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_l1_to_l2_messages_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot),
    toAppendOnlyTreeSnapshot(o.end_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot),
    toAppendOnlyTreeSnapshot(o.start_historic_blocks_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_historic_blocks_tree_snapshot),
    mapTuple(o.calldata_hash, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.l1_to_l2_messages_hash, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function fromRootRollupPublicInputs(o: RootRollupPublicInputs): MsgpackRootRollupPublicInputs {
  if (o.endAggregationObject === undefined) {
    throw new Error('Expected endAggregationObject in RootRollupPublicInputs serialization');
  }
  if (o.globalVariables === undefined) {
    throw new Error('Expected globalVariables in RootRollupPublicInputs serialization');
  }
  if (o.startPrivateDataTreeSnapshot === undefined) {
    throw new Error('Expected startPrivateDataTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.endPrivateDataTreeSnapshot === undefined) {
    throw new Error('Expected endPrivateDataTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.startNullifierTreeSnapshot === undefined) {
    throw new Error('Expected startNullifierTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.endNullifierTreeSnapshot === undefined) {
    throw new Error('Expected endNullifierTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.startContractTreeSnapshot === undefined) {
    throw new Error('Expected startContractTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.endContractTreeSnapshot === undefined) {
    throw new Error('Expected endContractTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.startPublicDataTreeRoot === undefined) {
    throw new Error('Expected startPublicDataTreeRoot in RootRollupPublicInputs serialization');
  }
  if (o.endPublicDataTreeRoot === undefined) {
    throw new Error('Expected endPublicDataTreeRoot in RootRollupPublicInputs serialization');
  }
  if (o.startTreeOfHistoricPrivateDataTreeRootsSnapshot === undefined) {
    throw new Error('Expected startTreeOfHistoricPrivateDataTreeRootsSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.endTreeOfHistoricPrivateDataTreeRootsSnapshot === undefined) {
    throw new Error('Expected endTreeOfHistoricPrivateDataTreeRootsSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.startTreeOfHistoricContractTreeRootsSnapshot === undefined) {
    throw new Error('Expected startTreeOfHistoricContractTreeRootsSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.endTreeOfHistoricContractTreeRootsSnapshot === undefined) {
    throw new Error('Expected endTreeOfHistoricContractTreeRootsSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.startL1ToL2MessagesTreeSnapshot === undefined) {
    throw new Error('Expected startL1ToL2MessagesTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.endL1ToL2MessagesTreeSnapshot === undefined) {
    throw new Error('Expected endL1ToL2MessagesTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.startTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot === undefined) {
    throw new Error(
      'Expected startTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot in RootRollupPublicInputs serialization',
    );
  }
  if (o.endTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot === undefined) {
    throw new Error(
      'Expected endTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot in RootRollupPublicInputs serialization',
    );
  }
  if (o.startHistoricBlocksTreeSnapshot === undefined) {
    throw new Error('Expected startHistoricBlocksTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.endHistoricBlocksTreeSnapshot === undefined) {
    throw new Error('Expected endHistoricBlocksTreeSnapshot in RootRollupPublicInputs serialization');
  }
  if (o.calldataHash === undefined) {
    throw new Error('Expected calldataHash in RootRollupPublicInputs serialization');
  }
  if (o.l1ToL2MessagesHash === undefined) {
    throw new Error('Expected l1ToL2MessagesHash in RootRollupPublicInputs serialization');
  }
  return {
    end_aggregation_object: fromNativeAggregationState(o.endAggregationObject),
    global_variables: fromGlobalVariables(o.globalVariables),
    start_private_data_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startPrivateDataTreeSnapshot),
    end_private_data_tree_snapshot: fromAppendOnlyTreeSnapshot(o.endPrivateDataTreeSnapshot),
    start_nullifier_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startNullifierTreeSnapshot),
    end_nullifier_tree_snapshot: fromAppendOnlyTreeSnapshot(o.endNullifierTreeSnapshot),
    start_contract_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startContractTreeSnapshot),
    end_contract_tree_snapshot: fromAppendOnlyTreeSnapshot(o.endContractTreeSnapshot),
    start_public_data_tree_root: toBuffer(o.startPublicDataTreeRoot),
    end_public_data_tree_root: toBuffer(o.endPublicDataTreeRoot),
    start_tree_of_historic_private_data_tree_roots_snapshot: fromAppendOnlyTreeSnapshot(
      o.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
    ),
    end_tree_of_historic_private_data_tree_roots_snapshot: fromAppendOnlyTreeSnapshot(
      o.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
    ),
    start_tree_of_historic_contract_tree_roots_snapshot: fromAppendOnlyTreeSnapshot(
      o.startTreeOfHistoricContractTreeRootsSnapshot,
    ),
    end_tree_of_historic_contract_tree_roots_snapshot: fromAppendOnlyTreeSnapshot(
      o.endTreeOfHistoricContractTreeRootsSnapshot,
    ),
    start_l1_to_l2_messages_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startL1ToL2MessagesTreeSnapshot),
    end_l1_to_l2_messages_tree_snapshot: fromAppendOnlyTreeSnapshot(o.endL1ToL2MessagesTreeSnapshot),
    start_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot: fromAppendOnlyTreeSnapshot(
      o.startTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot,
    ),
    end_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot: fromAppendOnlyTreeSnapshot(
      o.endTreeOfHistoricL1ToL2MessagesTreeRootsSnapshot,
    ),
    start_historic_blocks_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startHistoricBlocksTreeSnapshot),
    end_historic_blocks_tree_snapshot: fromAppendOnlyTreeSnapshot(o.endHistoricBlocksTreeSnapshot),
    calldata_hash: mapTuple(o.calldataHash, (v: Fr) => toBuffer(v)),
    l1_to_l2_messages_hash: mapTuple(o.l1ToL2MessagesHash, (v: Fr) => toBuffer(v)),
  };
}

export function abisComputeCompleteAddress(
  wasm: IWasmModule,
  arg0: Point,
  arg1: Fr,
  arg2: Fr,
  arg3: Fr,
): CompleteAddress {
  return toCompleteAddress(
    callCbind(wasm, 'abis__compute_complete_address', [
      fromPoint(arg0),
      toBuffer(arg1),
      toBuffer(arg2),
      toBuffer(arg3),
    ]),
  );
}
export function abisComputeCommitmentNonce(wasm: IWasmModule, arg0: Fr, arg1: Fr): Fr {
  return Fr.fromBuffer(callCbind(wasm, 'abis__compute_commitment_nonce', [toBuffer(arg0), toBuffer(arg1)]));
}
export function abisComputeUniqueCommitment(wasm: IWasmModule, arg0: Fr, arg1: Fr): Fr {
  return Fr.fromBuffer(callCbind(wasm, 'abis__compute_unique_commitment', [toBuffer(arg0), toBuffer(arg1)]));
}
export function abisSiloCommitment(wasm: IWasmModule, arg0: Address, arg1: Fr): Fr {
  return Fr.fromBuffer(callCbind(wasm, 'abis__silo_commitment', [toBuffer(arg0), toBuffer(arg1)]));
}
export function abisSiloNullifier(wasm: IWasmModule, arg0: Address, arg1: Fr): Fr {
  return Fr.fromBuffer(callCbind(wasm, 'abis__silo_nullifier', [toBuffer(arg0), toBuffer(arg1)]));
}
export function abisComputeBlockHash(
  wasm: IWasmModule,
  arg0: Fr,
  arg1: Fr,
  arg2: Fr,
  arg3: Fr,
  arg4: Fr,
  arg5: Fr,
): Fr {
  return Fr.fromBuffer(
    callCbind(wasm, 'abis__compute_block_hash', [
      toBuffer(arg0),
      toBuffer(arg1),
      toBuffer(arg2),
      toBuffer(arg3),
      toBuffer(arg4),
      toBuffer(arg5),
    ]),
  );
}
export function abisComputeBlockHashWithGlobals(
  wasm: IWasmModule,
  arg0: GlobalVariables,
  arg1: Fr,
  arg2: Fr,
  arg3: Fr,
  arg4: Fr,
  arg5: Fr,
): Fr {
  return Fr.fromBuffer(
    callCbind(wasm, 'abis__compute_block_hash_with_globals', [
      fromGlobalVariables(arg0),
      toBuffer(arg1),
      toBuffer(arg2),
      toBuffer(arg3),
      toBuffer(arg4),
      toBuffer(arg5),
    ]),
  );
}
export function abisComputeGlobalsHash(wasm: IWasmModule, arg0: GlobalVariables): Fr {
  return Fr.fromBuffer(callCbind(wasm, 'abis__compute_globals_hash', [fromGlobalVariables(arg0)]));
}
export function abisComputePublicDataTreeValue(wasm: IWasmModule, arg0: Fr): Fr {
  return Fr.fromBuffer(callCbind(wasm, 'abis__compute_public_data_tree_value', [toBuffer(arg0)]));
}
export function abisComputePublicDataTreeIndex(wasm: IWasmModule, arg0: Fr, arg1: Fr): Fr {
  return Fr.fromBuffer(callCbind(wasm, 'abis__compute_public_data_tree_index', [toBuffer(arg0), toBuffer(arg1)]));
}
export function privateKernelDummyPreviousKernel(wasm: IWasmModule): PreviousKernelData {
  return toPreviousKernelData(callCbind(wasm, 'private_kernel__dummy_previous_kernel', []));
}
export function privateKernelSimInit(
  wasm: IWasmModule,
  arg0: PrivateKernelInputsInit,
): CircuitError | KernelCircuitPublicInputs {
  return ((v: MsgpackCircuitError | MsgpackKernelCircuitPublicInputs) =>
    isCircuitError(v) ? toCircuitError(v) : toKernelCircuitPublicInputs(v))(
    callCbind(wasm, 'private_kernel__sim_init', [fromPrivateKernelInputsInit(arg0)]),
  );
}
export function privateKernelSimInner(
  wasm: IWasmModule,
  arg0: PrivateKernelInputsInner,
): CircuitError | KernelCircuitPublicInputs {
  return ((v: MsgpackCircuitError | MsgpackKernelCircuitPublicInputs) =>
    isCircuitError(v) ? toCircuitError(v) : toKernelCircuitPublicInputs(v))(
    callCbind(wasm, 'private_kernel__sim_inner', [fromPrivateKernelInputsInner(arg0)]),
  );
}
export function privateKernelSimOrdering(
  wasm: IWasmModule,
  arg0: PrivateKernelInputsOrdering,
): CircuitError | KernelCircuitPublicInputsFinal {
  return ((v: MsgpackCircuitError | MsgpackKernelCircuitPublicInputsFinal) =>
    isCircuitError(v) ? toCircuitError(v) : toKernelCircuitPublicInputsFinal(v))(
    callCbind(wasm, 'private_kernel__sim_ordering', [fromPrivateKernelInputsOrdering(arg0)]),
  );
}
export function publicKernelSim(wasm: IWasmModule, arg0: PublicKernelInputs): CircuitError | KernelCircuitPublicInputs {
  return ((v: MsgpackCircuitError | MsgpackKernelCircuitPublicInputs) =>
    isCircuitError(v) ? toCircuitError(v) : toKernelCircuitPublicInputs(v))(
    callCbind(wasm, 'public_kernel__sim', [fromPublicKernelInputs(arg0)]),
  );
}
export function baseRollupSim(wasm: IWasmModule, arg0: BaseRollupInputs): CircuitError | BaseOrMergeRollupPublicInputs {
  return ((v: MsgpackCircuitError | MsgpackBaseOrMergeRollupPublicInputs) =>
    isCircuitError(v) ? toCircuitError(v) : toBaseOrMergeRollupPublicInputs(v))(
    callCbind(wasm, 'base_rollup__sim', [fromBaseRollupInputs(arg0)]),
  );
}
export function mergeRollupSim(
  wasm: IWasmModule,
  arg0: MergeRollupInputs,
): CircuitError | BaseOrMergeRollupPublicInputs {
  return ((v: MsgpackCircuitError | MsgpackBaseOrMergeRollupPublicInputs) =>
    isCircuitError(v) ? toCircuitError(v) : toBaseOrMergeRollupPublicInputs(v))(
    callCbind(wasm, 'merge_rollup__sim', [fromMergeRollupInputs(arg0)]),
  );
}
export function rootRollupSim(wasm: IWasmModule, arg0: RootRollupInputs): CircuitError | RootRollupPublicInputs {
  return ((v: MsgpackCircuitError | MsgpackRootRollupPublicInputs) =>
    isCircuitError(v) ? toCircuitError(v) : toRootRollupPublicInputs(v))(
    callCbind(wasm, 'root_rollup__sim', [fromRootRollupInputs(arg0)]),
  );
}

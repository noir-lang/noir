/* eslint-disable camelcase,jsdoc/require-jsdoc */
// TODO: Remove this file as we no longer generate types from cpp.
import { Tuple, mapTuple } from '@aztec/foundation/serialize';
import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';
import mapValues from 'lodash.mapvalues';

import { CallRequest } from '../structs/call_request.js';
import { callCbind } from './cbind.js';
import {
  AppendOnlyTreeSnapshot,
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CircuitError,
  CombinedAccumulatedData,
  CombinedConstantData,
  ConstantRollupData,
  ContractDeploymentData,
  Fq,
  Fr,
  FunctionData,
  FunctionSelector,
  G1AffineElement,
  GlobalVariables,
  HistoricBlockData,
  KernelCircuitPublicInputs,
  MembershipWitness16,
  MembershipWitness20,
  NativeAggregationState,
  NewContractData,
  NullifierLeafPreimage,
  OptionallyRevealedData,
  Point,
  PreviousKernelData,
  PublicDataRead,
  PublicDataUpdateRequest,
  TxContext,
  VerificationKeyData,
  toBuffer,
} from './types.js';

interface MsgpackPoint {
  x: Buffer;
  y: Buffer;
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
  pending_read_requests: Tuple<Buffer, 128>;
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

export function fromCombinedAccumulatedData(o: CombinedAccumulatedData): MsgpackCombinedAccumulatedData {
  if (o.aggregationObject === undefined) {
    throw new Error('Expected aggregationObject in CombinedAccumulatedData serialization');
  }
  if (o.readRequests === undefined) {
    throw new Error('Expected readRequests in CombinedAccumulatedData serialization');
  }
  if (o.pendingReadRequests === undefined) {
    throw new Error('Expected pendingReadRequests in CombinedAccumulatedData serialization');
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
    pending_read_requests: mapTuple(o.pendingReadRequests, (v: Fr) => toBuffer(v)),
    new_commitments: mapTuple(o.newCommitments, (v: Fr) => toBuffer(v)),
    new_nullifiers: mapTuple(o.newNullifiers, (v: Fr) => toBuffer(v)),
    nullified_commitments: mapTuple(o.nullifiedCommitments, (v: Fr) => toBuffer(v)),
    private_call_stack: mapTuple(o.privateCallStack, (v: CallRequest) => toBuffer(v.hash)),
    public_call_stack: mapTuple(o.publicCallStack, (v: CallRequest) => toBuffer(v.hash)),
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
  note_hash_tree_root: Buffer;
  nullifier_tree_root: Buffer;
  contract_tree_root: Buffer;
  l1_to_l2_messages_tree_root: Buffer;
  blocks_tree_root: Buffer;
  private_kernel_vk_tree_root: Buffer;
  public_data_tree_root: Buffer;
  global_variables_hash: Buffer;
}

export function fromHistoricBlockData(o: HistoricBlockData): MsgpackHistoricBlockData {
  if (o.noteHashTreeRoot === undefined) {
    throw new Error('Expected noteHashTreeRoot in HistoricBlockData serialization');
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
    note_hash_tree_root: toBuffer(o.noteHashTreeRoot),
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

interface MsgpackMembershipWitness16 {
  leaf_index: Buffer;
  sibling_path: Tuple<Buffer, 16>;
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

interface MsgpackMembershipWitness20 {
  leaf_index: Buffer;
  sibling_path: Tuple<Buffer, 20>;
}

export function fromMembershipWitness20(o: MembershipWitness20): MsgpackMembershipWitness20 {
  if (o.leafIndex === undefined) {
    throw new Error('Expected leafIndex in MembershipWitness20 serialization');
  }
  if (o.siblingPath === undefined) {
    throw new Error('Expected siblingPath in MembershipWitness20 serialization');
  }
  return {
    leaf_index: toBuffer(o.leafIndex),
    sibling_path: mapTuple(o.siblingPath, (v: Fr) => toBuffer(v)),
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
  start_note_hash_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_nullifier_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_contract_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  start_public_data_tree_root: Buffer;
  start_historic_blocks_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  low_nullifier_leaf_preimages: Tuple<MsgpackNullifierLeafPreimage, 128>;
  low_nullifier_membership_witness: Tuple<MsgpackMembershipWitness20, 128>;
  new_commitments_subtree_sibling_path: Tuple<Buffer, 25>;
  new_nullifiers_subtree_sibling_path: Tuple<Buffer, 13>;
  new_contracts_subtree_sibling_path: Tuple<Buffer, 15>;
  new_public_data_update_requests_sibling_paths: Tuple<Tuple<Buffer, 254>, 32>;
  new_public_data_reads_sibling_paths: Tuple<Tuple<Buffer, 254>, 32>;
  historic_blocks_tree_root_membership_witnesses: Tuple<MsgpackMembershipWitness16, 2>;
  constants: MsgpackConstantRollupData;
}

export function fromBaseRollupInputs(o: BaseRollupInputs): MsgpackBaseRollupInputs {
  if (o.kernelData === undefined) {
    throw new Error('Expected kernelData in BaseRollupInputs serialization');
  }
  if (o.startNoteHashTreeSnapshot === undefined) {
    throw new Error('Expected startNoteHashTreeSnapshot in BaseRollupInputs serialization');
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
    start_note_hash_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startNoteHashTreeSnapshot),
    start_nullifier_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startNullifierTreeSnapshot),
    start_contract_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startContractTreeSnapshot),
    start_public_data_tree_root: toBuffer(o.startPublicDataTreeRoot),
    start_historic_blocks_tree_snapshot: fromAppendOnlyTreeSnapshot(o.startHistoricBlocksTreeSnapshot),
    low_nullifier_leaf_preimages: mapTuple(o.lowNullifierLeafPreimages, (v: NullifierLeafPreimage) =>
      fromNullifierLeafPreimage(v),
    ),
    low_nullifier_membership_witness: mapTuple(o.lowNullifierMembershipWitness, (v: MembershipWitness20) =>
      fromMembershipWitness20(v),
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
  start_note_hash_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
  end_note_hash_tree_snapshot: MsgpackAppendOnlyTreeSnapshot;
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
  if (o.start_note_hash_tree_snapshot === undefined) {
    throw new Error('Expected start_note_hash_tree_snapshot in BaseOrMergeRollupPublicInputs deserialization');
  }
  if (o.end_note_hash_tree_snapshot === undefined) {
    throw new Error('Expected end_note_hash_tree_snapshot in BaseOrMergeRollupPublicInputs deserialization');
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
    toAppendOnlyTreeSnapshot(o.start_note_hash_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_note_hash_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_nullifier_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_nullifier_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.start_contract_tree_snapshot),
    toAppendOnlyTreeSnapshot(o.end_contract_tree_snapshot),
    Fr.fromBuffer(o.start_public_data_tree_root),
    Fr.fromBuffer(o.end_public_data_tree_root),
    mapTuple(o.calldata_hash, (v: Buffer) => Fr.fromBuffer(v)),
  );
}

export function baseRollupSim(wasm: IWasmModule, arg0: BaseRollupInputs): CircuitError | BaseOrMergeRollupPublicInputs {
  return ((v: [number, MsgpackCircuitError | MsgpackBaseOrMergeRollupPublicInputs]) =>
    v[0] == 0
      ? toCircuitError(v[1] as MsgpackCircuitError)
      : toBaseOrMergeRollupPublicInputs(v[1] as MsgpackBaseOrMergeRollupPublicInputs))(
    callCbind(wasm, 'base_rollup__sim', [fromBaseRollupInputs(arg0)]),
  );
}

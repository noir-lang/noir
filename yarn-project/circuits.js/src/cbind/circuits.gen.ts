/* eslint-disable */
// GENERATED FILE DO NOT EDIT, RUN yarn remake-bindings
import { Buffer } from 'buffer';
import { callCbind } from './cbind.js';
import { IWasmModule } from '@aztec/foundation/wasm';
import {
  Fr,
  Address,
  Fq,
  G1AffineElement,
  NativeAggregationState,
  NewContractData,
  FunctionData,
  OptionallyRevealedData,
  PublicDataUpdateRequest,
  PublicDataRead,
  CombinedAccumulatedData,
  PrivateHistoricTreeRoots,
  CombinedHistoricTreeRoots,
  ContractDeploymentData,
  TxContext,
  CombinedConstantData,
  KernelCircuitPublicInputs,
  Proof,
  VerificationKeyData,
  PreviousKernelData,
  CallContext,
  ContractStorageUpdateRequest,
  ContractStorageRead,
  PublicCircuitPublicInputs,
  PublicCallStackItem,
  PublicCallData,
  PublicKernelInputs,
  CircuitError,
  isCircuitError,
} from './types.js';
import { Tuple, mapTuple } from '@aztec/foundation/serialize';
import mapValues from 'lodash.mapvalues';
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
    x: o.x.toBuffer(),
    y: o.y.toBuffer(),
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
    public_inputs: o.publicInputs.map((v: Fr) => v.toBuffer()),
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
    contract_address: o.contractAddress.toBuffer(),
    portal_contract_address: o.portalContractAddress.toBuffer(),
    function_tree_root: o.functionTreeRoot.toBuffer(),
  };
}

interface MsgpackFunctionData {
  function_selector: number;
  is_private: boolean;
  is_constructor: boolean;
}

export function toFunctionData(o: MsgpackFunctionData): FunctionData {
  if (o.function_selector === undefined) {
    throw new Error('Expected function_selector in FunctionData deserialization');
  }
  if (o.is_private === undefined) {
    throw new Error('Expected is_private in FunctionData deserialization');
  }
  if (o.is_constructor === undefined) {
    throw new Error('Expected is_constructor in FunctionData deserialization');
  }
  return new FunctionData(o.function_selector, o.is_private, o.is_constructor);
}

export function fromFunctionData(o: FunctionData): MsgpackFunctionData {
  if (o.functionSelector === undefined) {
    throw new Error('Expected functionSelector in FunctionData serialization');
  }
  if (o.isPrivate === undefined) {
    throw new Error('Expected isPrivate in FunctionData serialization');
  }
  if (o.isConstructor === undefined) {
    throw new Error('Expected isConstructor in FunctionData serialization');
  }
  return {
    function_selector: o.functionSelector,
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
    call_stack_item_hash: o.callStackItemHash.toBuffer(),
    function_data: fromFunctionData(o.functionData),
    vk_hash: o.vkHash.toBuffer(),
    portal_contract_address: o.portalContractAddress.toBuffer(),
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
    leaf_index: o.leafIndex.toBuffer(),
    old_value: o.oldValue.toBuffer(),
    new_value: o.newValue.toBuffer(),
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
    leaf_index: o.leafIndex.toBuffer(),
    value: o.value.toBuffer(),
  };
}

interface MsgpackCombinedAccumulatedData {
  aggregation_object: MsgpackNativeAggregationState;
  new_commitments: Tuple<Buffer, 4>;
  new_nullifiers: Tuple<Buffer, 4>;
  private_call_stack: Tuple<Buffer, 8>;
  public_call_stack: Tuple<Buffer, 8>;
  new_l2_to_l1_msgs: Tuple<Buffer, 2>;
  encrypted_logs_hash: Tuple<Buffer, 2>;
  unencrypted_logs_hash: Tuple<Buffer, 2>;
  encrypted_log_preimages_length: Buffer;
  unencrypted_log_preimages_length: Buffer;
  new_contracts: Tuple<MsgpackNewContractData, 1>;
  optionally_revealed_data: Tuple<MsgpackOptionallyRevealedData, 4>;
  public_data_update_requests: Tuple<MsgpackPublicDataUpdateRequest, 4>;
  public_data_reads: Tuple<MsgpackPublicDataRead, 4>;
}

export function toCombinedAccumulatedData(o: MsgpackCombinedAccumulatedData): CombinedAccumulatedData {
  if (o.aggregation_object === undefined) {
    throw new Error('Expected aggregation_object in CombinedAccumulatedData deserialization');
  }
  if (o.new_commitments === undefined) {
    throw new Error('Expected new_commitments in CombinedAccumulatedData deserialization');
  }
  if (o.new_nullifiers === undefined) {
    throw new Error('Expected new_nullifiers in CombinedAccumulatedData deserialization');
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
    mapTuple(o.new_commitments, (v: Buffer) => Fr.fromBuffer(v)),
    mapTuple(o.new_nullifiers, (v: Buffer) => Fr.fromBuffer(v)),
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
  if (o.newCommitments === undefined) {
    throw new Error('Expected newCommitments in CombinedAccumulatedData serialization');
  }
  if (o.newNullifiers === undefined) {
    throw new Error('Expected newNullifiers in CombinedAccumulatedData serialization');
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
    new_commitments: mapTuple(o.newCommitments, (v: Fr) => v.toBuffer()),
    new_nullifiers: mapTuple(o.newNullifiers, (v: Fr) => v.toBuffer()),
    private_call_stack: mapTuple(o.privateCallStack, (v: Fr) => v.toBuffer()),
    public_call_stack: mapTuple(o.publicCallStack, (v: Fr) => v.toBuffer()),
    new_l2_to_l1_msgs: mapTuple(o.newL2ToL1Msgs, (v: Fr) => v.toBuffer()),
    encrypted_logs_hash: mapTuple(o.encryptedLogsHash, (v: Fr) => v.toBuffer()),
    unencrypted_logs_hash: mapTuple(o.unencryptedLogsHash, (v: Fr) => v.toBuffer()),
    encrypted_log_preimages_length: o.encryptedLogPreimagesLength.toBuffer(),
    unencrypted_log_preimages_length: o.unencryptedLogPreimagesLength.toBuffer(),
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

interface MsgpackPrivateHistoricTreeRoots {
  private_data_tree_root: Buffer;
  nullifier_tree_root: Buffer;
  contract_tree_root: Buffer;
  l1_to_l2_messages_tree_root: Buffer;
  private_kernel_vk_tree_root: Buffer;
}

export function toPrivateHistoricTreeRoots(o: MsgpackPrivateHistoricTreeRoots): PrivateHistoricTreeRoots {
  if (o.private_data_tree_root === undefined) {
    throw new Error('Expected private_data_tree_root in PrivateHistoricTreeRoots deserialization');
  }
  if (o.nullifier_tree_root === undefined) {
    throw new Error('Expected nullifier_tree_root in PrivateHistoricTreeRoots deserialization');
  }
  if (o.contract_tree_root === undefined) {
    throw new Error('Expected contract_tree_root in PrivateHistoricTreeRoots deserialization');
  }
  if (o.l1_to_l2_messages_tree_root === undefined) {
    throw new Error('Expected l1_to_l2_messages_tree_root in PrivateHistoricTreeRoots deserialization');
  }
  if (o.private_kernel_vk_tree_root === undefined) {
    throw new Error('Expected private_kernel_vk_tree_root in PrivateHistoricTreeRoots deserialization');
  }
  return new PrivateHistoricTreeRoots(
    Fr.fromBuffer(o.private_data_tree_root),
    Fr.fromBuffer(o.nullifier_tree_root),
    Fr.fromBuffer(o.contract_tree_root),
    Fr.fromBuffer(o.l1_to_l2_messages_tree_root),
    Fr.fromBuffer(o.private_kernel_vk_tree_root),
  );
}

export function fromPrivateHistoricTreeRoots(o: PrivateHistoricTreeRoots): MsgpackPrivateHistoricTreeRoots {
  if (o.privateDataTreeRoot === undefined) {
    throw new Error('Expected privateDataTreeRoot in PrivateHistoricTreeRoots serialization');
  }
  if (o.nullifierTreeRoot === undefined) {
    throw new Error('Expected nullifierTreeRoot in PrivateHistoricTreeRoots serialization');
  }
  if (o.contractTreeRoot === undefined) {
    throw new Error('Expected contractTreeRoot in PrivateHistoricTreeRoots serialization');
  }
  if (o.l1ToL2MessagesTreeRoot === undefined) {
    throw new Error('Expected l1ToL2MessagesTreeRoot in PrivateHistoricTreeRoots serialization');
  }
  if (o.privateKernelVkTreeRoot === undefined) {
    throw new Error('Expected privateKernelVkTreeRoot in PrivateHistoricTreeRoots serialization');
  }
  return {
    private_data_tree_root: o.privateDataTreeRoot.toBuffer(),
    nullifier_tree_root: o.nullifierTreeRoot.toBuffer(),
    contract_tree_root: o.contractTreeRoot.toBuffer(),
    l1_to_l2_messages_tree_root: o.l1ToL2MessagesTreeRoot.toBuffer(),
    private_kernel_vk_tree_root: o.privateKernelVkTreeRoot.toBuffer(),
  };
}

interface MsgpackCombinedHistoricTreeRoots {
  private_historic_tree_roots: MsgpackPrivateHistoricTreeRoots;
}

export function toCombinedHistoricTreeRoots(o: MsgpackCombinedHistoricTreeRoots): CombinedHistoricTreeRoots {
  if (o.private_historic_tree_roots === undefined) {
    throw new Error('Expected private_historic_tree_roots in CombinedHistoricTreeRoots deserialization');
  }
  return new CombinedHistoricTreeRoots(toPrivateHistoricTreeRoots(o.private_historic_tree_roots));
}

export function fromCombinedHistoricTreeRoots(o: CombinedHistoricTreeRoots): MsgpackCombinedHistoricTreeRoots {
  if (o.privateHistoricTreeRoots === undefined) {
    throw new Error('Expected privateHistoricTreeRoots in CombinedHistoricTreeRoots serialization');
  }
  return {
    private_historic_tree_roots: fromPrivateHistoricTreeRoots(o.privateHistoricTreeRoots),
  };
}

interface MsgpackContractDeploymentData {
  deployer_public_key: Tuple<Buffer, 2>;
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
    mapTuple(o.deployer_public_key, (v: Buffer) => Fr.fromBuffer(v)),
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
    deployer_public_key: mapTuple(o.deployerPublicKey, (v: Fr) => v.toBuffer()),
    constructor_vk_hash: o.constructorVkHash.toBuffer(),
    function_tree_root: o.functionTreeRoot.toBuffer(),
    contract_address_salt: o.contractAddressSalt.toBuffer(),
    portal_contract_address: o.portalContractAddress.toBuffer(),
  };
}

interface MsgpackTxContext {
  is_fee_payment_tx: boolean;
  is_rebate_payment_tx: boolean;
  is_contract_deployment_tx: boolean;
  contract_deployment_data: MsgpackContractDeploymentData;
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
  return new TxContext(
    o.is_fee_payment_tx,
    o.is_rebate_payment_tx,
    o.is_contract_deployment_tx,
    toContractDeploymentData(o.contract_deployment_data),
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
  return {
    is_fee_payment_tx: o.isFeePaymentTx,
    is_rebate_payment_tx: o.isRebatePaymentTx,
    is_contract_deployment_tx: o.isContractDeploymentTx,
    contract_deployment_data: fromContractDeploymentData(o.contractDeploymentData),
  };
}

interface MsgpackCombinedConstantData {
  historic_tree_roots: MsgpackCombinedHistoricTreeRoots;
  tx_context: MsgpackTxContext;
}

export function toCombinedConstantData(o: MsgpackCombinedConstantData): CombinedConstantData {
  if (o.historic_tree_roots === undefined) {
    throw new Error('Expected historic_tree_roots in CombinedConstantData deserialization');
  }
  if (o.tx_context === undefined) {
    throw new Error('Expected tx_context in CombinedConstantData deserialization');
  }
  return new CombinedConstantData(toCombinedHistoricTreeRoots(o.historic_tree_roots), toTxContext(o.tx_context));
}

export function fromCombinedConstantData(o: CombinedConstantData): MsgpackCombinedConstantData {
  if (o.historicTreeRoots === undefined) {
    throw new Error('Expected historicTreeRoots in CombinedConstantData serialization');
  }
  if (o.txContext === undefined) {
    throw new Error('Expected txContext in CombinedConstantData serialization');
  }
  return {
    historic_tree_roots: fromCombinedHistoricTreeRoots(o.historicTreeRoots),
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
  composer_type: number;
  circuit_size: number;
  num_public_inputs: number;
  commitments: Record<string, MsgpackG1AffineElement>;
  contains_recursive_proof: boolean;
  recursive_proof_public_input_indices: number[];
}

export function toVerificationKeyData(o: MsgpackVerificationKeyData): VerificationKeyData {
  if (o.composer_type === undefined) {
    throw new Error('Expected composer_type in VerificationKeyData deserialization');
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
    o.composer_type,
    o.circuit_size,
    o.num_public_inputs,
    mapValues(o.commitments, (v: MsgpackG1AffineElement) => toG1AffineElement(v)),
    o.contains_recursive_proof,
    o.recursive_proof_public_input_indices.map((v: number) => v),
  );
}

export function fromVerificationKeyData(o: VerificationKeyData): MsgpackVerificationKeyData {
  if (o.composerType === undefined) {
    throw new Error('Expected composerType in VerificationKeyData serialization');
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
    composer_type: o.composerType,
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
    vk_path: mapTuple(o.vkPath, (v: Fr) => v.toBuffer()),
  };
}

interface MsgpackCallContext {
  msg_sender: Buffer;
  storage_contract_address: Buffer;
  portal_contract_address: Buffer;
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
    msg_sender: o.msgSender.toBuffer(),
    storage_contract_address: o.storageContractAddress.toBuffer(),
    portal_contract_address: o.portalContractAddress.toBuffer(),
    is_delegate_call: o.isDelegateCall,
    is_static_call: o.isStaticCall,
    is_contract_deployment: o.isContractDeployment,
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
    storage_slot: o.storageSlot.toBuffer(),
    old_value: o.oldValue.toBuffer(),
    new_value: o.newValue.toBuffer(),
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
    storage_slot: o.storageSlot.toBuffer(),
    current_value: o.currentValue.toBuffer(),
  };
}

interface MsgpackPublicCircuitPublicInputs {
  call_context: MsgpackCallContext;
  args_hash: Buffer;
  return_values: Tuple<Buffer, 4>;
  contract_storage_update_requests: Tuple<MsgpackContractStorageUpdateRequest, 4>;
  contract_storage_reads: Tuple<MsgpackContractStorageRead, 4>;
  public_call_stack: Tuple<Buffer, 4>;
  new_commitments: Tuple<Buffer, 4>;
  new_nullifiers: Tuple<Buffer, 4>;
  new_l2_to_l1_msgs: Tuple<Buffer, 2>;
  historic_public_data_tree_root: Buffer;
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
  if (o.historic_public_data_tree_root === undefined) {
    throw new Error('Expected historic_public_data_tree_root in PublicCircuitPublicInputs deserialization');
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
    Fr.fromBuffer(o.historic_public_data_tree_root),
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
  if (o.historicPublicDataTreeRoot === undefined) {
    throw new Error('Expected historicPublicDataTreeRoot in PublicCircuitPublicInputs serialization');
  }
  if (o.proverAddress === undefined) {
    throw new Error('Expected proverAddress in PublicCircuitPublicInputs serialization');
  }
  return {
    call_context: fromCallContext(o.callContext),
    args_hash: o.argsHash.toBuffer(),
    return_values: mapTuple(o.returnValues, (v: Fr) => v.toBuffer()),
    contract_storage_update_requests: mapTuple(o.contractStorageUpdateRequests, (v: ContractStorageUpdateRequest) =>
      fromContractStorageUpdateRequest(v),
    ),
    contract_storage_reads: mapTuple(o.contractStorageReads, (v: ContractStorageRead) => fromContractStorageRead(v)),
    public_call_stack: mapTuple(o.publicCallStack, (v: Fr) => v.toBuffer()),
    new_commitments: mapTuple(o.newCommitments, (v: Fr) => v.toBuffer()),
    new_nullifiers: mapTuple(o.newNullifiers, (v: Fr) => v.toBuffer()),
    new_l2_to_l1_msgs: mapTuple(o.newL2ToL1Msgs, (v: Fr) => v.toBuffer()),
    historic_public_data_tree_root: o.historicPublicDataTreeRoot.toBuffer(),
    prover_address: o.proverAddress.toBuffer(),
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
    contract_address: o.contractAddress.toBuffer(),
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
    portal_contract_address: o.portalContractAddress.toBuffer(),
    bytecode_hash: o.bytecodeHash.toBuffer(),
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

export function abisComputeContractAddress(
  wasm: IWasmModule,
  arg0: Tuple<Fr, 2>,
  arg1: Fr,
  arg2: Fr,
  arg3: Fr,
): Address {
  return Address.fromBuffer(
    callCbind(wasm, 'abis__compute_contract_address', [
      mapTuple(arg0, (v: Fr) => v.toBuffer()),
      arg1.toBuffer(),
      arg2.toBuffer(),
      arg3.toBuffer(),
    ]),
  );
}
export function abisSiloCommitment(wasm: IWasmModule, arg0: Address, arg1: Fr): Fr {
  return Fr.fromBuffer(callCbind(wasm, 'abis__silo_commitment', [arg0.toBuffer(), arg1.toBuffer()]));
}
export function privateKernelDummyPreviousKernel(wasm: IWasmModule): PreviousKernelData {
  return toPreviousKernelData(callCbind(wasm, 'private_kernel__dummy_previous_kernel', []));
}
export function publicKernelSim(wasm: IWasmModule, arg0: PublicKernelInputs): CircuitError | KernelCircuitPublicInputs {
  return ((v: MsgpackCircuitError | MsgpackKernelCircuitPublicInputs) =>
    isCircuitError(v) ? toCircuitError(v) : toKernelCircuitPublicInputs(v))(
    callCbind(wasm, 'public_kernel__sim', [fromPublicKernelInputs(arg0)]),
  );
}

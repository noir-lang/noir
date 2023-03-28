import { assertLength, FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import {
  CONTRACT_TREE_HEIGHT,
  EMITTED_EVENTS_LENGTH,
  FUNCTION_TREE_HEIGHT,
  KERNEL_L1_MSG_STACK_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  PRIVATE_CALL_STACK_LENGTH,
  VK_TREE_HEIGHT,
} from './constants.js';
import { FunctionData } from './function_data.js';
import { PrivateCallStackItem } from './private_call_stack_item.js';
import { AggregationObject, AztecAddress, EthAddress, Fr, MembershipWitness, UInt32, UInt8Vector } from './shared.js';
import { SignedTxRequest, TxContext } from './tx.js';
import { VerificationKey } from './verification_key.js';

export class OldTreeRoots {
  constructor(
    public privateDataTreeRoot: Fr,
    public nullifierTreeRoot: Fr,
    public contractTreeRoot: Fr,
    public privateKernelVkTreeRoot: Fr, // future enhancement
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.privateDataTreeRoot,
      this.nullifierTreeRoot,
      this.contractTreeRoot,
      this.privateKernelVkTreeRoot,
    );
  }
}

export class ConstantData {
  constructor(public oldTreeRoots: OldTreeRoots, public txContext: TxContext) {}

  toBuffer() {
    return serializeToBuffer(this.oldTreeRoots, this.txContext);
  }
}

// Not to be confused with ContractDeploymentData (maybe think of better names)
export class NewContractData {
  constructor(
    public contractAddress: AztecAddress,
    public portalContractAddress: EthAddress,
    public functionTreeRoot: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.portalContractAddress, this.functionTreeRoot);
  }
}

export class OptionallyRevealedData {
  constructor(
    public callStackItemHash: Fr,
    public functionData: FunctionData,
    public emittedEvents: Fr[],
    public vkHash: Fr,
    public portalContractAddress: EthAddress,
    public payFeeFromL1: boolean,
    public payFeeFromPublicL2: boolean,
    public calledFromL1: boolean,
    public calledFromPublicL2: boolean,
  ) {
    assertLength(this, 'emittedEvents', EMITTED_EVENTS_LENGTH);
  }

  toBuffer() {
    return serializeToBuffer(
      this.callStackItemHash,
      this.functionData,
      this.emittedEvents,
      this.vkHash,
      this.portalContractAddress,
      this.payFeeFromL1,
      this.payFeeFromPublicL2,
      this.calledFromL1,
      this.calledFromPublicL2,
    );
  }
}

export class AccumulatedData {
  constructor(
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations

    public privateCallCount: Fr,

    public newCommitments: Fr[],
    public newNullifiers: Fr[],

    public privateCallStack: Fr[],
    public publicCallStack: Fr[],
    public l1MsgStack: Fr[],

    public newContracts: NewContractData[],

    public optionallyRevealedData: OptionallyRevealedData[],
  ) {
    assertLength(this, 'newCommitments', KERNEL_NEW_COMMITMENTS_LENGTH);
    assertLength(this, 'newNullifiers', KERNEL_NEW_NULLIFIERS_LENGTH);
    assertLength(this, 'privateCallStack', KERNEL_PRIVATE_CALL_STACK_LENGTH);
    assertLength(this, 'publicCallStack', KERNEL_PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, 'l1MsgStack', KERNEL_L1_MSG_STACK_LENGTH);
    assertLength(this, 'newContracts', KERNEL_NEW_CONTRACTS_LENGTH);
    assertLength(this, 'optionallyRevealedData', KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH);
  }

  toBuffer() {
    return serializeToBuffer(
      this.aggregationObject,
      this.privateCallCount,
      this.newCommitments,
      this.newNullifiers,
      this.privateCallStack,
      this.publicCallStack,
      this.l1MsgStack,
      this.newContracts,
      this.optionallyRevealedData,
    );
  }
}

export class PrivateKernelPublicInputs {
  constructor(public end: AccumulatedData, public constants: ConstantData, public isPrivateKernel: true) {}

  toBuffer() {
    return serializeToBuffer(this.end, this.constants, this.isPrivateKernel);
  }
}

export class PreviousKernelData {
  constructor(
    public publicInputs: PrivateKernelPublicInputs,
    public proof: UInt8Vector,
    public vk: VerificationKey,
    public vkIndex: UInt32, // the index of the kernel circuit's vk in a big tree of kernel circuit vks
    public vkSiblingPath: Fr[],
  ) {
    assertLength(this, 'vkSiblingPath', VK_TREE_HEIGHT);
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.publicInputs, this.proof, this.vk, this.vkIndex, this.vkSiblingPath);
  }
}

/**
 * Private call data.
 * @see cpp/src/aztec3/circuits/abis/call_stack_item.hpp.
 */
export class PrivateCallData {
  constructor(
    public callStackItem: PrivateCallStackItem,
    public privateCallStackPreimages: PrivateCallStackItem[],
    public proof: UInt8Vector,
    public vk: VerificationKey,
    public functionLeafMembershipWitness: MembershipWitness<typeof FUNCTION_TREE_HEIGHT>,
    public contractLeafMembershipWitness: MembershipWitness<typeof CONTRACT_TREE_HEIGHT>,
    public portalContractAddress: EthAddress,
  ) {
    assertLength(this, 'privateCallStackPreimages', PRIVATE_CALL_STACK_LENGTH);
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<PrivateCallData>) {
    return [
      // NOTE: Must have same order as CPP.
      fields.callStackItem,
      fields.privateCallStackPreimages,
      fields.proof,
      fields.vk,
      fields.functionLeafMembershipWitness,
      fields.contractLeafMembershipWitness,
      fields.portalContractAddress,
    ] as const;
  }

  static from(fields: FieldsOf<PrivateCallData>): PrivateCallData {
    return new PrivateCallData(...PrivateCallData.getFields(fields));
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(...PrivateCallData.getFields(this));
  }
}

/**
 * Input to the private kernel circuit.
 */
export class PrivateKernelInputs {
  constructor(
    public signedTxRequest: SignedTxRequest,
    public previousKernel: PreviousKernelData,
    public privateCall: PrivateCallData,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.signedTxRequest, this.previousKernel, this.privateCall);
  }
}

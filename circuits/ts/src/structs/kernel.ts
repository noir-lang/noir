import { assertLength, checkLength } from "../utils/jsUtils.js";
import { serializeToBuffer } from "../utils/serialize.js";
import {
  KERNEL_L1_MSG_STACK_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  VK_TREE_HEIGHT,
  EMITTED_EVENTS_LENGTH,
} from "./constants.js";
import { FunctionData } from "./function_data.js";
import {
  AggregationObject,
  AztecAddress,
  EthAddress,
  Fr,
  UInt8Vector,
  UInt32,
} from "./shared.js";
import { TxContext } from "./tx.js";
import { VerificationKey } from "./verification_key.js";

export class OldTreeRoots {
  constructor(
    public privateDataTreeRoot: Fr,
    public nullifierTreeRoot: Fr,
    public contractTreeRoot: Fr,
    public privateKernelVkTreeRoot: Fr // future enhancement
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.privateDataTreeRoot,
      this.nullifierTreeRoot,
      this.contractTreeRoot,
      this.privateKernelVkTreeRoot
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
    public functionTreeRoot: Fr
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.contractAddress,
      this.portalContractAddress,
      this.functionTreeRoot
    );
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
    public calledFromPublicL2: boolean
  ) {
    assertLength(this, "emittedEvents", EMITTED_EVENTS_LENGTH);
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
      this.calledFromPublicL2
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

    public optionallyRevealedData: OptionallyRevealedData[]
  ) {
    assertLength(this, "newCommitments", KERNEL_NEW_COMMITMENTS_LENGTH);
    assertLength(this, "newNullifiers", KERNEL_NEW_NULLIFIERS_LENGTH);
    assertLength(this, "privateCallStack", KERNEL_PRIVATE_CALL_STACK_LENGTH);
    assertLength(this, "publicCallStack", KERNEL_PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, "l1MsgStack", KERNEL_L1_MSG_STACK_LENGTH);
    assertLength(this, "newContracts", KERNEL_NEW_CONTRACTS_LENGTH);
    assertLength(
      this,
      "optionallyRevealedData",
      KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH
    );
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
      this.optionallyRevealedData
    );
  }
}

export class PrivateKernelPublicInputs {
  constructor(
    public end: AccumulatedData,
    public constants: ConstantData,
    public isPrivateKernel: true
  ) {}

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
    public vkSiblingPath: Fr[]
  ) {
    checkLength(this.vkSiblingPath, VK_TREE_HEIGHT, "vkSiblingPath");
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.publicInputs,
      this.proof,
      this.vk,
      this.vkIndex,
      this.vkSiblingPath
    );
  }
}

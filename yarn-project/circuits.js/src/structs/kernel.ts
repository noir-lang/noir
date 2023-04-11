import { CircuitsWasm, getDummyPreviousKernelData } from '../index.js';
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
import { AggregationObject, MembershipWitness, UInt32, UInt8Vector } from './shared.js';
import { ContractDeploymentData, SignedTxRequest, TxContext } from './tx.js';
import { VerificationKey } from './verification_key.js';
import { AztecAddress, EthAddress, Fr, BufferReader } from '@aztec/foundation';
import times from 'lodash.times';

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

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): OldTreeRoots {
    const reader = BufferReader.asReader(buffer);
    return new OldTreeRoots(reader.readFr(), reader.readFr(), reader.readFr(), reader.readFr());
  }
}

export class ConstantData {
  constructor(public oldTreeRoots: OldTreeRoots, public txContext: TxContext) {}

  toBuffer() {
    return serializeToBuffer(this.oldTreeRoots, this.txContext);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): ConstantData {
    const reader = BufferReader.asReader(buffer);
    return new ConstantData(reader.readObject(OldTreeRoots), reader.readObject(TxContext));
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

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): NewContractData {
    const reader = BufferReader.asReader(buffer);
    return new NewContractData(reader.readObject(AztecAddress), new EthAddress(reader.readBytes(32)), reader.readFr());
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

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): OptionallyRevealedData {
    const reader = BufferReader.asReader(buffer);
    return new OptionallyRevealedData(
      reader.readFr(),
      reader.readObject(FunctionData),
      reader.readArray(EMITTED_EVENTS_LENGTH, Fr),
      reader.readFr(),
      new EthAddress(reader.readBytes(32)),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readBoolean(),
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

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): AccumulatedData {
    const reader = BufferReader.asReader(buffer);
    return new AccumulatedData(
      reader.readObject(AggregationObject),
      reader.readFr(),
      reader.readArray(KERNEL_NEW_COMMITMENTS_LENGTH, Fr),
      reader.readArray(KERNEL_NEW_NULLIFIERS_LENGTH, Fr),
      reader.readArray(KERNEL_PRIVATE_CALL_STACK_LENGTH, Fr),
      reader.readArray(KERNEL_PUBLIC_CALL_STACK_LENGTH, Fr),
      reader.readArray(KERNEL_L1_MSG_STACK_LENGTH, Fr),
      reader.readArray(KERNEL_NEW_CONTRACTS_LENGTH, NewContractData),
      reader.readArray(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, OptionallyRevealedData),
    );
  }
}

export class PrivateKernelPublicInputs {
  constructor(public end: AccumulatedData, public constants: ConstantData, public isPrivateKernel: true) {}

  toBuffer() {
    return serializeToBuffer(this.end, this.constants, this.isPrivateKernel);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelPublicInputs(reader.readObject(AccumulatedData), reader.readObject(ConstantData), true);
  }

  static makeEmpty() {
    return new PrivateKernelPublicInputs(makeEmptyAccumulatedData(), makeEmptyConstantData(), true);
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

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PreviousKernelData {
    const reader = BufferReader.asReader(buffer);
    return new PreviousKernelData(
      reader.readObject(PrivateKernelPublicInputs),
      reader.readObject(UInt8Vector),
      reader.readObject(VerificationKey),
      reader.readNumber(),
      reader.readArray(VK_TREE_HEIGHT, Fr),
    );
  }

  /**
   * Creates an empty instance, valid enough to be accepted by circuits
   */
  static makeEmpty() {
    return new PreviousKernelData(
      PrivateKernelPublicInputs.makeEmpty(),
      makeEmptyProof(),
      VerificationKey.makeFake(),
      0,
      Array(VK_TREE_HEIGHT).fill(frZero()),
    );
  }
}

export class DummyPreviousKernelData {
  private static instance: DummyPreviousKernelData;

  private constructor(private data: PreviousKernelData) {}

  public static async getDummyPreviousKernelData(wasm: CircuitsWasm) {
    if (!DummyPreviousKernelData.instance) {
      const data = await getDummyPreviousKernelData(wasm);
      DummyPreviousKernelData.instance = new DummyPreviousKernelData(data);
    }

    return DummyPreviousKernelData.instance.getData();
  }

  public getData() {
    return this.data;
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

// Helper functions for making empty structs (delete them eventually to use real data or factories instances)
// or move them somewhere generic, or within each struct

function frZero() {
  return Fr.fromBuffer(Buffer.alloc(32, 0));
}

function makeEmptyEthAddress() {
  return new EthAddress(Buffer.alloc(20, 0));
}

function makeEmptyNewContractData(): NewContractData {
  return new NewContractData(AztecAddress.ZERO, makeEmptyEthAddress(), frZero());
}

function makeEmptyTxContext(): TxContext {
  const deploymentData = new ContractDeploymentData(frZero(), frZero(), frZero(), makeEmptyEthAddress());
  return new TxContext(false, false, true, deploymentData);
}

function makeEmptyOldTreeRoots(): OldTreeRoots {
  return new OldTreeRoots(frZero(), frZero(), frZero(), frZero());
}

function makeEmptyConstantData(): ConstantData {
  return new ConstantData(makeEmptyOldTreeRoots(), makeEmptyTxContext());
}

function makeEmptyOptionallyRevealedData(): OptionallyRevealedData {
  return new OptionallyRevealedData(
    frZero(),
    new FunctionData(Buffer.alloc(4), true, true),
    times(EMITTED_EVENTS_LENGTH, frZero),
    frZero(),
    makeEmptyEthAddress(),
    false,
    false,
    false,
    false,
  );
}

function makeEmptyAccumulatedData(): AccumulatedData {
  return new AccumulatedData(
    AggregationObject.makeFake(),
    frZero(),
    times(KERNEL_NEW_COMMITMENTS_LENGTH, frZero),
    times(KERNEL_NEW_NULLIFIERS_LENGTH, frZero),
    times(KERNEL_PRIVATE_CALL_STACK_LENGTH, frZero),
    times(KERNEL_PUBLIC_CALL_STACK_LENGTH, frZero),
    times(KERNEL_L1_MSG_STACK_LENGTH, frZero),
    times(KERNEL_NEW_CONTRACTS_LENGTH, makeEmptyNewContractData),
    times(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, makeEmptyOptionallyRevealedData),
  );
}

export function makeEmptyProof() {
  return new UInt8Vector(Buffer.alloc(0));
}

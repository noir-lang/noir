import times from 'lodash.times';
import { assertLength } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { AggregationObject } from '../aggregation_object.js';
import {
  EMITTED_EVENTS_LENGTH,
  KERNEL_NEW_L2_TO_L1_MSGS_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_DATA_READS_LENGTH,
  KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
} from '../constants.js';
import { FunctionData } from '../function_data.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

/**
 * The information assembled after the contract deployment was processed by the private kernel circuit.
 *
 * Note: Not to be confused with `ContractDeploymentData`.
 */
export class NewContractData {
  constructor(
    /**
     * Aztec address of the contract.
     */
    public contractAddress: AztecAddress,
    /**
     * Ethereum address of the portal contract on L1.
     */
    public portalContractAddress: EthAddress,
    /**
     * Function tree root of the contract.
     */
    public functionTreeRoot: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.portalContractAddress, this.functionTreeRoot);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized `NewContractData`.
   */
  static fromBuffer(buffer: Buffer | BufferReader): NewContractData {
    const reader = BufferReader.asReader(buffer);
    return new NewContractData(reader.readObject(AztecAddress), new EthAddress(reader.readBytes(32)), reader.readFr());
  }

  static empty() {
    return new NewContractData(AztecAddress.ZERO, EthAddress.ZERO, Fr.ZERO);
  }
}

/**
 * Info which a user might want to reveal to the world.
 * Note: Currently not used (2023-05-12).
 */
export class OptionallyRevealedData {
  constructor(
    /**
     * Hash of the call stack item from which this info was originates.
     */
    public callStackItemHash: Fr,
    /**
     * Function data of a function call from which this info originates.
     */
    public functionData: FunctionData,
    /**
     * Events emitted by the function call from which this info originates.
     */
    public emittedEvents: Fr[],
    /**
     * Verification key hash of the function call from which this info originates.
     */
    public vkHash: Fr,
    /**
     * Address of the portal contract corresponding to the L2 contract on which the function above was invoked.
     */
    public portalContractAddress: EthAddress,
    /**
     * Whether the fee was paid from the L1 account of the user.
     */
    public payFeeFromL1: boolean,
    /**
     * Whether the fee was paid from a public account on L2.
     */
    public payFeeFromPublicL2: boolean,
    /**
     * Whether the function call was invoked from L1.
     */
    public calledFromL1: boolean,
    /**
     * Whether the function call was invoked from the public L2 account of the user.
     */
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
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized OptionallyRevealedData.
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

  static empty() {
    return new OptionallyRevealedData(
      Fr.ZERO,
      FunctionData.empty(),
      times(EMITTED_EVENTS_LENGTH, Fr.zero),
      Fr.ZERO,
      EthAddress.ZERO,
      false,
      false,
      false,
      false,
    );
  }
}

/**
 * Read operations from the public state tree.
 */
export class PublicDataRead {
  constructor(
    /**
     * Index of the leaf in the public data tree.
     */
    public readonly leafIndex: Fr,
    /**
     * Returned value from the public data tree.
     */
    public readonly value: Fr,
  ) {}

  static from(args: {
    /**
     * Index of the leaf in the public data tree.
     */
    leafIndex: Fr;
    /**
     * Returned value from the public data tree.
     */
    value: Fr;
  }) {
    return new PublicDataRead(args.leafIndex, args.value);
  }

  toBuffer() {
    return serializeToBuffer(this.leafIndex, this.value);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataRead(reader.readFr(), reader.readFr());
  }

  static empty() {
    return new PublicDataRead(Fr.ZERO, Fr.ZERO);
  }

  toFriendlyJSON() {
    return `Leaf=${this.leafIndex.toFriendlyJSON()}: ${this.value.toFriendlyJSON()}`;
  }
}

/**
 * Write operations on the public data tree including the previous value.
 */
export class PublicDataUpdateRequest {
  constructor(
    /**
     * Index of the leaf in the public data tree which is to be updated.
     */
    public readonly leafIndex: Fr,
    /**
     * Old value of the leaf.
     */
    public readonly oldValue: Fr,
    /**
     * New value of the leaf.
     */
    public readonly newValue: Fr,
  ) {}

  static from(args: {
    /**
     * Index of the leaf in the public data tree which is to be updated.
     */
    leafIndex: Fr;
    /**
     * Old value of the leaf.
     */
    oldValue: Fr;
    /**
     * New value of the leaf.
     */
    newValue: Fr;
  }) {
    return new PublicDataUpdateRequest(args.leafIndex, args.oldValue, args.newValue);
  }

  toBuffer() {
    return serializeToBuffer(this.leafIndex, this.oldValue, this.newValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataUpdateRequest(reader.readFr(), reader.readFr(), reader.readFr());
  }

  static empty() {
    return new PublicDataUpdateRequest(Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }

  toFriendlyJSON() {
    return `Leaf=${this.leafIndex.toFriendlyJSON()}: ${this.oldValue.toFriendlyJSON()} => ${this.newValue.toFriendlyJSON()}`;
  }
}

/**
 * Data that is accumulated during the execution of the transaction.
 */
export class CombinedAccumulatedData {
  constructor(
    /**
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject,
    /**
     * The number of new commitments made in this transaction.
     */
    public newCommitments: Fr[],
    /**
     * The number of new nullifiers made in this transaction.
     */
    public newNullifiers: Fr[],
    /**
     * Current private call stack.
     */
    public privateCallStack: Fr[],
    /**
     * Current public call stack.
     */
    public publicCallStack: Fr[],
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public newL2ToL1Msgs: Fr[],
    /**
     * All the new contracts deployed in this transaction.
     */
    public newContracts: NewContractData[],

    /**
     * All the optionally revealed data in this transaction.
     */
    public optionallyRevealedData: OptionallyRevealedData[],

    /**
     * All the public data update requests made in this transaction.
     */
    public publicDataUpdateRequests: PublicDataUpdateRequest[],
    /**
     * All the public data reads made in this transaction.
     */
    public publicDataReads: PublicDataRead[],
  ) {
    assertLength(this, 'newCommitments', KERNEL_NEW_COMMITMENTS_LENGTH);
    assertLength(this, 'newNullifiers', KERNEL_NEW_NULLIFIERS_LENGTH);
    assertLength(this, 'privateCallStack', KERNEL_PRIVATE_CALL_STACK_LENGTH);
    assertLength(this, 'publicCallStack', KERNEL_PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, 'newL2ToL1Msgs', KERNEL_NEW_L2_TO_L1_MSGS_LENGTH);
    assertLength(this, 'newContracts', KERNEL_NEW_CONTRACTS_LENGTH);
    assertLength(this, 'optionallyRevealedData', KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH);
    assertLength(this, 'publicDataUpdateRequests', KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH);
    assertLength(this, 'publicDataReads', KERNEL_PUBLIC_DATA_READS_LENGTH);
  }

  toBuffer() {
    return serializeToBuffer(
      this.aggregationObject,
      this.newCommitments,
      this.newNullifiers,
      this.privateCallStack,
      this.publicCallStack,
      this.newL2ToL1Msgs,
      this.newContracts,
      this.optionallyRevealedData,
      this.publicDataUpdateRequests,
      this.publicDataReads,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedAccumulatedData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedAccumulatedData(
      reader.readObject(AggregationObject),
      reader.readArray(KERNEL_NEW_COMMITMENTS_LENGTH, Fr),
      reader.readArray(KERNEL_NEW_NULLIFIERS_LENGTH, Fr),
      reader.readArray(KERNEL_PRIVATE_CALL_STACK_LENGTH, Fr),
      reader.readArray(KERNEL_PUBLIC_CALL_STACK_LENGTH, Fr),
      reader.readArray(KERNEL_NEW_L2_TO_L1_MSGS_LENGTH, Fr),
      reader.readArray(KERNEL_NEW_CONTRACTS_LENGTH, NewContractData),
      reader.readArray(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, OptionallyRevealedData),
      reader.readArray(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH, PublicDataUpdateRequest),
      reader.readArray(KERNEL_PUBLIC_DATA_READS_LENGTH, PublicDataRead),
    );
  }

  static empty() {
    return new CombinedAccumulatedData(
      AggregationObject.makeFake(),
      times(KERNEL_NEW_COMMITMENTS_LENGTH, Fr.zero),
      times(KERNEL_NEW_NULLIFIERS_LENGTH, Fr.zero),
      times(KERNEL_PRIVATE_CALL_STACK_LENGTH, Fr.zero),
      times(KERNEL_PUBLIC_CALL_STACK_LENGTH, Fr.zero),
      times(KERNEL_NEW_L2_TO_L1_MSGS_LENGTH, Fr.zero),
      times(KERNEL_NEW_CONTRACTS_LENGTH, NewContractData.empty),
      times(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, OptionallyRevealedData.empty),
      times(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH, PublicDataUpdateRequest.empty),
      times(KERNEL_PUBLIC_DATA_READS_LENGTH, PublicDataRead.empty),
    );
  }
}

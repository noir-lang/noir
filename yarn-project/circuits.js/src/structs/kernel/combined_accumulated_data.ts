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

  static empty() {
    return new NewContractData(AztecAddress.ZERO, EthAddress.ZERO, Fr.ZERO);
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
  constructor(public readonly leafIndex: Fr, public readonly value: Fr) {}

  static from(args: { leafIndex: Fr; value: Fr }) {
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
}

/**
 * Write operations on the public state tree including the previous value.
 */
export class PublicDataUpdateRequest {
  constructor(public readonly leafIndex: Fr, public readonly oldValue: Fr, public readonly newValue: Fr) {}

  static from(args: { leafIndex: Fr; oldValue: Fr; newValue: Fr }) {
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
}

export class CombinedAccumulatedData {
  constructor(
    public aggregationObject: AggregationObject,

    public privateCallCount: Fr,
    public publicCallCount: Fr,

    public newCommitments: Fr[],
    public newNullifiers: Fr[],

    public privateCallStack: Fr[],
    public publicCallStack: Fr[],
    public newL2ToL1Msgs: Fr[],

    public newContracts: NewContractData[],

    public optionallyRevealedData: OptionallyRevealedData[],

    public publicDataUpdateRequests: PublicDataUpdateRequest[],
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
      this.privateCallCount,
      this.publicCallCount,
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
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedAccumulatedData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedAccumulatedData(
      reader.readObject(AggregationObject),
      reader.readFr(),
      reader.readFr(),
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
      Fr.ZERO,
      Fr.ZERO,
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

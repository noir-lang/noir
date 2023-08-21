import { BufferReader, Tuple } from '@aztec/foundation/serialize';

import {
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_READ_REQUESTS_PER_TX,
  NUM_FIELDS_PER_SHA256,
} from '../../cbind/constants.gen.js';
import { assertMemberLength, makeTuple } from '../../index.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import {
  AggregationObject,
  AztecAddress,
  EthAddress,
  Fr,
  FunctionData,
  ReadRequestMembershipWitness,
} from '../index.js';

/**
 * The information assembled after the contract deployment was processed by the private kernel circuit.
 *
 * Note: Not to be confused with `ContractDeploymentData`.
 */
export class NewContractData {
  /**
   * Ethereum address of the portal contract on L1.
   */
  public portalContractAddress: EthAddress;
  constructor(
    /**
     * Aztec address of the contract.
     */
    public contractAddress: AztecAddress,
    /**
     * Ethereum address of the portal contract on L1.
     * TODO(AD): refactor this later
     * currently there is a kludge with circuits cpp as it emits an AztecAddress
     */
    portalContractAddress: EthAddress | AztecAddress,
    /**
     * Function tree root of the contract.
     */
    public functionTreeRoot: Fr,
  ) {
    // Handle circuits emitting this as an AztecAddress
    this.portalContractAddress = new EthAddress(portalContractAddress.toBuffer());
  }

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
  /**
   * Address of the portal contract corresponding to the L2 contract on which the function above was invoked.
   *
   * TODO(AD): refactor this later
   * currently there is a kludge with circuits cpp as it emits an AztecAddress
   */
  public portalContractAddress: EthAddress;
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
     * Verification key hash of the function call from which this info originates.
     */
    public vkHash: Fr,
    /**
     * Address of the portal contract corresponding to the L2 contract on which the function above was invoked.
     *
     * TODO(AD): refactor this later
     * currently there is a kludge with circuits cpp as it emits an AztecAddress
     */
    portalContractAddress: EthAddress | AztecAddress,
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
    // Handle circuits emitting this as an AztecAddress
    this.portalContractAddress = EthAddress.fromField(portalContractAddress.toField());
  }

  toBuffer() {
    return serializeToBuffer(
      this.callStackItemHash,
      this.functionData,
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
    /**
     * Optional side effect counter tracking position of this event in tx execution.
     */
    public readonly sideEffectCounter?: number,
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
    /**
     * Optional side effect counter tracking position of this event in tx execution.
     */
    public readonly sideEffectCounter?: number,
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
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
    /**
     * All the read requests made in this transaction.
     */
    public readRequests: Tuple<Fr, typeof MAX_READ_REQUESTS_PER_TX>,
    /**
     * All the read request membership witnesses made in this transaction.
     */
    public readRequestMembershipWitnesses: Tuple<ReadRequestMembershipWitness, typeof MAX_READ_REQUESTS_PER_TX>,
    /**
     * The new commitments made in this transaction.
     */
    public newCommitments: Tuple<Fr, typeof MAX_NEW_COMMITMENTS_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The commitments which are nullified by a nullifier in the above list. For pending nullifiers, we have:
     * nullifiedCommitments[j] != 0 if and only if newNullifiers[j] nullifies nullifiedCommitments[j]
     */
    public nullifiedCommitments: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Current private call stack.
     */
    public privateCallStack: Tuple<Fr, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    /**
     * Current public call stack.
     */
    public publicCallStack: Tuple<Fr, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    /**
     * Accumulated encrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public encryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public unencryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Total accumulated length of the encrypted log preimages emitted in all the previous kernel iterations
     */
    public encryptedLogPreimagesLength: Fr,
    /**
     * Total accumulated length of the unencrypted log preimages emitted in all the previous kernel iterations
     */
    public unencryptedLogPreimagesLength: Fr,
    /**
     * All the new contracts deployed in this transaction.
     */
    public newContracts: Tuple<NewContractData, typeof MAX_NEW_CONTRACTS_PER_TX>,
    /**
     * All the optionally revealed data in this transaction.
     */
    public optionallyRevealedData: Tuple<OptionallyRevealedData, typeof MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX>,
    /**
     * All the public data update requests made in this transaction.
     */
    public publicDataUpdateRequests: Tuple<PublicDataUpdateRequest, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    /**
     * All the public data reads made in this transaction.
     */
    public publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>,
  ) {
    assertMemberLength(this, 'readRequests', MAX_READ_REQUESTS_PER_TX);
    assertMemberLength(this, 'readRequestMembershipWitnesses', MAX_READ_REQUESTS_PER_TX);
    assertMemberLength(this, 'newCommitments', MAX_NEW_COMMITMENTS_PER_TX);
    assertMemberLength(this, 'newNullifiers', MAX_NEW_NULLIFIERS_PER_TX);
    assertMemberLength(this, 'nullifiedCommitments', MAX_NEW_NULLIFIERS_PER_TX);
    assertMemberLength(this, 'privateCallStack', MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX);
    assertMemberLength(this, 'publicCallStack', MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX);
    assertMemberLength(this, 'newL2ToL1Msgs', MAX_NEW_L2_TO_L1_MSGS_PER_TX);
    assertMemberLength(this, 'encryptedLogsHash', NUM_FIELDS_PER_SHA256);
    assertMemberLength(this, 'unencryptedLogsHash', NUM_FIELDS_PER_SHA256);
    assertMemberLength(this, 'newContracts', MAX_NEW_CONTRACTS_PER_TX);
    assertMemberLength(this, 'optionallyRevealedData', MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX);
    assertMemberLength(this, 'publicDataUpdateRequests', MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX);
    assertMemberLength(this, 'publicDataReads', MAX_PUBLIC_DATA_READS_PER_TX);
  }

  toBuffer() {
    return serializeToBuffer(
      this.aggregationObject,
      this.readRequests,
      this.readRequestMembershipWitnesses,
      this.newCommitments,
      this.newNullifiers,
      this.nullifiedCommitments,
      this.privateCallStack,
      this.publicCallStack,
      this.newL2ToL1Msgs,
      this.encryptedLogsHash,
      this.unencryptedLogsHash,
      this.encryptedLogPreimagesLength,
      this.unencryptedLogPreimagesLength,
      this.newContracts,
      this.optionallyRevealedData,
      this.publicDataUpdateRequests,
      this.publicDataReads,
    );
  }

  toString() {
    return this.toBuffer().toString();
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
      reader.readArray(MAX_READ_REQUESTS_PER_TX, Fr),
      reader.readArray(MAX_READ_REQUESTS_PER_TX, ReadRequestMembershipWitness),
      reader.readArray(MAX_NEW_COMMITMENTS_PER_TX, Fr),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Fr),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Fr),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, Fr),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, Fr),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      reader.readArray(2, Fr),
      reader.readArray(2, Fr),
      reader.readFr(),
      reader.readFr(),
      reader.readArray(MAX_NEW_CONTRACTS_PER_TX, NewContractData),
      reader.readArray(MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX, OptionallyRevealedData),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readArray(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return CombinedAccumulatedData.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new CombinedAccumulatedData(
      AggregationObject.makeFake(),
      makeTuple(MAX_READ_REQUESTS_PER_TX, Fr.zero),
      makeTuple(MAX_READ_REQUESTS_PER_TX, () => ReadRequestMembershipWitness.empty(BigInt(0))),
      makeTuple(MAX_NEW_COMMITMENTS_PER_TX, Fr.zero),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Fr.zero),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Fr.zero),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, Fr.zero),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, Fr.zero),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      makeTuple(2, Fr.zero),
      makeTuple(2, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_NEW_CONTRACTS_PER_TX, NewContractData.empty),
      makeTuple(MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX, OptionallyRevealedData.empty),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
    );
  }
}

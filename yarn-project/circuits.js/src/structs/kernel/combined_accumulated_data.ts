import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_COMMITMENTS_PER_TX_META,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX_META,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX_META,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_READ_REQUESTS_PER_TX,
  NUM_FIELDS_PER_SHA256,
} from '../../constants.gen.js';
import { CallRequest } from '../call_request.js';
import { NullifierKeyValidationRequestContext } from '../nullifier_key_validation_request.js';
import { SideEffect, SideEffectLinkedToNoteHash } from '../side_effects.js';

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
     * Contract class id.
     */
    public contractClassId: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.portalContractAddress, this.contractClassId);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized `NewContractData`.
   */
  static fromBuffer(buffer: Buffer | BufferReader): NewContractData {
    const reader = BufferReader.asReader(buffer);
    return new NewContractData(reader.readObject(AztecAddress), reader.readObject(EthAddress), Fr.fromBuffer(reader));
  }

  static empty() {
    return new NewContractData(AztecAddress.ZERO, EthAddress.ZERO, Fr.ZERO);
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
    public readonly leafSlot: Fr,
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
    return serializeToBuffer(this.leafSlot, this.value);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataRead(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  static empty() {
    return new PublicDataRead(Fr.ZERO, Fr.ZERO);
  }

  toFriendlyJSON() {
    return `Leaf=${this.leafSlot.toFriendlyJSON()}: ${this.value.toFriendlyJSON()}`;
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
    public readonly leafSlot: Fr,
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
    return serializeToBuffer(this.leafSlot, this.oldValue, this.newValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataUpdateRequest(Fr.fromBuffer(reader), Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  static empty() {
    return new PublicDataUpdateRequest(Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }

  toFriendlyJSON() {
    return `Leaf=${this.leafSlot.toFriendlyJSON()}: ${this.oldValue.toFriendlyJSON()} => ${this.newValue.toFriendlyJSON()}`;
  }
}

/**
 * Data that is accumulated during the execution of the transaction.
 */
export class CombinedAccumulatedData {
  constructor(
    /**
     * All the read requests made in this transaction.
     */
    public readRequests: Tuple<SideEffect, typeof MAX_READ_REQUESTS_PER_TX>,
    /**
     * All the nullifier key validation requests made in this transaction.
     */
    public nullifierKeyValidationRequests: Tuple<
      NullifierKeyValidationRequestContext,
      typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
    >,
    /**
     * The new commitments made in this transaction.
     */
    public newCommitments: Tuple<SideEffect, typeof MAX_NEW_COMMITMENTS_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Current private call stack.
     */
    public privateCallStack: Tuple<CallRequest, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    /**
     * Current public call stack.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
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
     * All the public data update requests made in this transaction.
     */
    public publicDataUpdateRequests: Tuple<PublicDataUpdateRequest, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    /**
     * All the public data reads made in this transaction.
     */
    public publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.readRequests,
      this.nullifierKeyValidationRequests,
      this.newCommitments,
      this.newNullifiers,
      this.privateCallStack,
      this.publicCallStack,
      this.newL2ToL1Msgs,
      this.encryptedLogsHash,
      this.unencryptedLogsHash,
      this.encryptedLogPreimagesLength,
      this.unencryptedLogPreimagesLength,
      this.newContracts,
      this.publicDataUpdateRequests,
      this.publicDataReads,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedAccumulatedData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedAccumulatedData(
      reader.readArray(MAX_READ_REQUESTS_PER_TX, SideEffect),
      reader.readArray(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, NullifierKeyValidationRequestContext),
      reader.readArray(MAX_NEW_COMMITMENTS_PER_TX, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      reader.readArray(2, Fr),
      reader.readArray(2, Fr),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_NEW_CONTRACTS_PER_TX, NewContractData),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readArray(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead),
    );
  }

  static fromFinalAccumulatedData(finalData: FinalAccumulatedData): CombinedAccumulatedData {
    return new CombinedAccumulatedData(
      makeTuple(MAX_READ_REQUESTS_PER_TX, SideEffect.empty),
      makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, NullifierKeyValidationRequestContext.empty),
      finalData.newCommitments,
      finalData.newNullifiers,
      finalData.privateCallStack,
      finalData.publicCallStack,
      finalData.newL2ToL1Msgs,
      finalData.encryptedLogsHash,
      finalData.unencryptedLogsHash,
      finalData.encryptedLogPreimagesLength,
      finalData.unencryptedLogPreimagesLength,
      finalData.newContracts,
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
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
      makeTuple(MAX_READ_REQUESTS_PER_TX, SideEffect.empty),
      makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, NullifierKeyValidationRequestContext.empty),
      makeTuple(MAX_NEW_COMMITMENTS_PER_TX, SideEffect.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      makeTuple(2, Fr.zero),
      makeTuple(2, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_NEW_CONTRACTS_PER_TX, NewContractData.empty),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
    );
  }
}

/**
 * Specific accumulated data structure for the final ordering private kernel circuit. It is included
 *  in the final public inputs of private kernel circuit.
 */
export class FinalAccumulatedData {
  constructor(
    /**
     * The new commitments made in this transaction.
     */
    public newCommitments: Tuple<SideEffect, typeof MAX_NEW_COMMITMENTS_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Current private call stack.
     * TODO(#3417): Given this field must empty, should we just remove it?
     */
    public privateCallStack: Tuple<CallRequest, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    /**
     * Current public call stack.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
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
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.newCommitments,
      this.newNullifiers,
      this.privateCallStack,
      this.publicCallStack,
      this.newL2ToL1Msgs,
      this.encryptedLogsHash,
      this.unencryptedLogsHash,
      this.encryptedLogPreimagesLength,
      this.unencryptedLogPreimagesLength,
      this.newContracts,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FinalAccumulatedData {
    const reader = BufferReader.asReader(buffer);
    return new FinalAccumulatedData(
      reader.readArray(MAX_NEW_COMMITMENTS_PER_TX, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      reader.readArray(2, Fr),
      reader.readArray(2, Fr),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_NEW_CONTRACTS_PER_TX, NewContractData),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return FinalAccumulatedData.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new FinalAccumulatedData(
      makeTuple(MAX_NEW_COMMITMENTS_PER_TX, SideEffect.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      makeTuple(2, Fr.zero),
      makeTuple(2, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_NEW_CONTRACTS_PER_TX, NewContractData.empty),
    );
  }
}

export class AccumulatedMetaData {
  constructor(
    /**
     * The new commitments made in this transaction.
     */
    public newCommitments: Tuple<SideEffect, typeof MAX_NEW_COMMITMENTS_PER_TX_META>,
    /**
     * The new nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX_META>,
    /**
     * Current public call stack.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX_META>,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.newCommitments, this.newNullifiers, this.publicCallStack);
  }

  static fromBuffer(buffer: Buffer | BufferReader): AccumulatedMetaData {
    const reader = BufferReader.asReader(buffer);
    return new AccumulatedMetaData(
      reader.readArray(MAX_NEW_COMMITMENTS_PER_TX_META, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX_META, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX_META, CallRequest),
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static fromString(str: string) {
    return AccumulatedMetaData.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new AccumulatedMetaData(
      makeTuple(MAX_NEW_COMMITMENTS_PER_TX_META, SideEffect.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX_META, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX_META, CallRequest.empty),
    );
  }
}

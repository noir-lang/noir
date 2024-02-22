import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { isArrayEmpty } from '@aztec/foundation/collection';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, Tuple, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import {
  GeneratorIndex,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NOTE_HASHES_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  NUM_FIELDS_PER_SHA256,
  PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH,
  RETURN_VALUES_LENGTH,
} from '../constants.gen.js';
import { CallContext } from './call_context.js';
import { ContractStorageRead } from './contract_storage_read.js';
import { ContractStorageUpdateRequest } from './contract_storage_update_request.js';
import { Header } from './header.js';
import { L2ToL1Message } from './l2_to_l1_message.js';
import { SideEffect, SideEffectLinkedToNoteHash } from './side_effects.js';

/**
 * Public inputs to a public circuit.
 */
export class PublicCircuitPublicInputs {
  constructor(
    /**
     * Current call context.
     */
    public callContext: CallContext,
    /**
     * Pedersen hash of the arguments of the call.
     */
    public argsHash: Fr,
    /**
     * Return values of the call.
     */
    public returnValues: Tuple<Fr, typeof RETURN_VALUES_LENGTH>,
    /**
     * Contract storage update requests executed during the call.
     */
    public contractStorageUpdateRequests: Tuple<
      ContractStorageUpdateRequest,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL
    >,
    /**
     * Contract storage reads executed during the call.
     */
    public contractStorageReads: Tuple<ContractStorageRead, typeof MAX_PUBLIC_DATA_READS_PER_CALL>,
    /**
     * Public call stack of the current kernel iteration.
     */
    public publicCallStackHashes: Tuple<Fr, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    /**
     * New note hashes created within a public execution call
     */
    public newNoteHashes: Tuple<SideEffect, typeof MAX_NEW_NOTE_HASHES_PER_CALL>,
    /**
     * New nullifiers created within a public execution call
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_CALL>,
    /**
     * New L2 to L1 messages generated during the call.
     */
    public newL2ToL1Msgs: Tuple<L2ToL1Message, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    /**
     * Hash of the unencrypted logs emitted in this function call.
     * Note: Represented as an array of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public unencryptedLogsHash: [Fr, Fr],
    /**
     * Length of the unencrypted log preimages emitted in this function call.
     */
    public unencryptedLogPreimagesLength: Fr,
    /**
     * Header of a block whose state is used during public execution. Set by sequencer to be a header of a block
     * previous to the one in which the tx is included.
     */
    public historicalHeader: Header,
    /**
     * Address of the prover.
     */
    public proverAddress: AztecAddress,
  ) {}

  /**
   * Create PublicCircuitPublicInputs from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A PublicCircuitPublicInputs object.
   */
  static from(fields: FieldsOf<PublicCircuitPublicInputs>): PublicCircuitPublicInputs {
    return new PublicCircuitPublicInputs(...PublicCircuitPublicInputs.getFields(fields));
  }

  /**
   * Returns an empty instance.
   * @returns An empty instance.
   */
  public static empty() {
    return new PublicCircuitPublicInputs(
      CallContext.empty(),
      Fr.ZERO,
      makeTuple(RETURN_VALUES_LENGTH, Fr.zero),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, ContractStorageUpdateRequest.empty),
      makeTuple(MAX_PUBLIC_DATA_READS_PER_CALL, ContractStorageRead.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, Fr.zero),
      makeTuple(MAX_NEW_NOTE_HASHES_PER_CALL, SideEffect.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, L2ToL1Message.empty),
      makeTuple(2, Fr.zero),
      Fr.ZERO,
      Header.empty(),
      AztecAddress.ZERO,
    );
  }

  isEmpty() {
    const isSideEffectArrayEmpty = (arr: SideEffect[]) => isArrayEmpty(arr, item => item.isEmpty());
    const isSideEffectLinkedArrayEmpty = (arr: SideEffectLinkedToNoteHash[]) =>
      isArrayEmpty(arr, item => item.isEmpty());
    const isFrArrayEmpty = (arr: Fr[]) => isArrayEmpty(arr, item => item.isZero());
    return (
      this.callContext.isEmpty() &&
      this.argsHash.isZero() &&
      isFrArrayEmpty(this.returnValues) &&
      isArrayEmpty(this.contractStorageUpdateRequests, item => item.isEmpty()) &&
      isArrayEmpty(this.contractStorageReads, item => item.isEmpty()) &&
      isFrArrayEmpty(this.publicCallStackHashes) &&
      isSideEffectArrayEmpty(this.newNoteHashes) &&
      isSideEffectLinkedArrayEmpty(this.newNullifiers) &&
      isArrayEmpty(this.newL2ToL1Msgs, item => item.isEmpty()) &&
      isFrArrayEmpty(this.unencryptedLogsHash) &&
      this.unencryptedLogPreimagesLength.isZero() &&
      this.historicalHeader.isEmpty() &&
      this.proverAddress.isZero()
    );
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<PublicCircuitPublicInputs>) {
    return [
      fields.callContext,
      fields.argsHash,
      fields.returnValues,
      fields.contractStorageUpdateRequests,
      fields.contractStorageReads,
      fields.publicCallStackHashes,
      fields.newNoteHashes,
      fields.newNullifiers,
      fields.newL2ToL1Msgs,
      fields.unencryptedLogsHash,
      fields.unencryptedLogPreimagesLength,
      fields.historicalHeader,
      fields.proverAddress,
    ] as const;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(...PublicCircuitPublicInputs.getFields(this));
  }

  toFields(): Fr[] {
    const fields = serializeToFields(...PublicCircuitPublicInputs.getFields(this));
    if (fields.length !== PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH) {
      throw new Error(
        `Invalid number of fields for PublicCircuitPublicInputs. Expected ${PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PublicCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new PublicCircuitPublicInputs(
      reader.readObject(CallContext),
      reader.readObject(Fr),
      reader.readArray(RETURN_VALUES_LENGTH, Fr),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, ContractStorageUpdateRequest),
      reader.readArray(MAX_PUBLIC_DATA_READS_PER_CALL, ContractStorageRead),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, Fr),
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_CALL, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_CALL, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, L2ToL1Message),
      reader.readArray(NUM_FIELDS_PER_SHA256, Fr),
      reader.readObject(Fr),
      reader.readObject(Header),
      reader.readObject(AztecAddress),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): PublicCircuitPublicInputs {
    const reader = FieldReader.asReader(fields);

    return new PublicCircuitPublicInputs(
      CallContext.fromFields(reader),
      reader.readField(),
      reader.readFieldArray(RETURN_VALUES_LENGTH),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, ContractStorageUpdateRequest),
      reader.readArray(MAX_PUBLIC_DATA_READS_PER_CALL, ContractStorageRead),
      reader.readFieldArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL),
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_CALL, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_CALL, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, L2ToL1Message),
      reader.readFieldArray(NUM_FIELDS_PER_SHA256),
      reader.readField(),
      Header.fromFields(reader),
      AztecAddress.fromFields(reader),
    );
  }

  hash(): Fr {
    return pedersenHash(
      this.toFields().map(field => field.toBuffer()),
      GeneratorIndex.PUBLIC_CIRCUIT_PUBLIC_INPUTS,
    );
  }
}

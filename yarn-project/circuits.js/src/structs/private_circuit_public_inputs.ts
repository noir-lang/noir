import { makeTuple } from '@aztec/foundation/array';
import { isArrayEmpty } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import {
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_READ_REQUESTS_PER_CALL,
  NUM_FIELDS_PER_SHA256,
  RETURN_VALUES_LENGTH,
} from '../constants.gen.js';
import { CallContext } from './call_context.js';
import { BlockHeader, SideEffect, SideEffectLinkedToNoteHash } from './index.js';
import { NullifierKeyValidationRequest } from './nullifier_key_validation_request.js';
import { ContractDeploymentData } from './tx_context.js';

/**
 * Public inputs to a private circuit.
 * @see abis/private_circuit_public_inputs.hpp.
 */
export class PrivateCircuitPublicInputs {
  constructor(
    /**
     * Context of the call corresponding to this private circuit execution.
     */
    public callContext: CallContext,
    /**
     * Pedersen hash of function arguments.
     */
    public argsHash: Fr,
    /**
     * Return values of the corresponding function call.
     */
    public returnValues: Tuple<Fr, typeof RETURN_VALUES_LENGTH>,
    /**
     * Read requests created by the corresponding function call.
     */
    public readRequests: Tuple<SideEffect, typeof MAX_READ_REQUESTS_PER_CALL>,
    /**
     * Nullifier key validation requests created by the corresponding function call.
     */
    public nullifierKeyValidationRequests: Tuple<
      NullifierKeyValidationRequest,
      typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL
    >,
    /**
     * New commitments created by the corresponding function call.
     */
    public newCommitments: Tuple<SideEffect, typeof MAX_NEW_COMMITMENTS_PER_CALL>,
    /**
     * New nullifiers created by the corresponding function call.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_CALL>,
    /**
     * Private call stack at the current kernel iteration.
     */
    public privateCallStackHashes: Tuple<Fr, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    /**
     * Public call stack at the current kernel iteration.
     */
    public publicCallStackHashes: Tuple<Fr, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    /**
     * New L2 to L1 messages created by the corresponding function call.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    /**
     * The end side effect counter for this call.
     */
    public endSideEffectCounter: Fr,
    /**
     * Hash of the encrypted logs emitted in this function call.
     * Note: Represented as an array of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public encryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Hash of the unencrypted logs emitted in this function call.
     * Note: Represented as an array of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public unencryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Length of the encrypted log preimages emitted in this function call.
     * Note: Here so that the gas cost of this request can be measured by circuits, without actually needing to feed
     *       in the variable-length data.
     */
    public encryptedLogPreimagesLength: Fr,
    /**
     * Length of the unencrypted log preimages emitted in this function call.
     */
    public unencryptedLogPreimagesLength: Fr,
    /**
     * Historical roots of the data trees, used to calculate the block hash the user is proving against.
     */
    public blockHeader: BlockHeader,
    /**
     * Deployment data of contracts being deployed in this kernel iteration.
     */
    public contractDeploymentData: ContractDeploymentData,
    /**
     * Chain Id of the instance.
     */
    public chainId: Fr,
    /**
     * Version of the instance.
     */
    public version: Fr,
  ) {}

  /**
   * Create PrivateCircuitPublicInputs from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A PrivateCircuitPublicInputs object.
   */
  static from(fields: FieldsOf<PrivateCircuitPublicInputs>): PrivateCircuitPublicInputs {
    return new PrivateCircuitPublicInputs(...PrivateCircuitPublicInputs.getFields(fields));
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateCircuitPublicInputs(
      reader.readObject(CallContext),
      reader.readObject(Fr),
      reader.readArray(RETURN_VALUES_LENGTH, Fr),
      reader.readArray(MAX_READ_REQUESTS_PER_CALL, SideEffect),
      reader.readArray(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL, NullifierKeyValidationRequest),
      reader.readArray(MAX_NEW_COMMITMENTS_PER_CALL, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_CALL, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, Fr),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, Fr),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, Fr),
      reader.readObject(Fr),
      reader.readArray(NUM_FIELDS_PER_SHA256, Fr),
      reader.readArray(NUM_FIELDS_PER_SHA256, Fr),
      reader.readObject(Fr),
      reader.readObject(Fr),
      reader.readObject(BlockHeader),
      reader.readObject(ContractDeploymentData),
      reader.readObject(Fr),
      reader.readObject(Fr),
    );
  }

  /**
   * Create an empty PrivateCircuitPublicInputs.
   * @returns An empty PrivateCircuitPublicInputs object.
   */
  public static empty(): PrivateCircuitPublicInputs {
    return new PrivateCircuitPublicInputs(
      CallContext.empty(),
      Fr.ZERO,
      makeTuple(RETURN_VALUES_LENGTH, Fr.zero),
      makeTuple(MAX_READ_REQUESTS_PER_CALL, SideEffect.empty),
      makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL, NullifierKeyValidationRequest.empty),
      makeTuple(MAX_NEW_COMMITMENTS_PER_CALL, SideEffect.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, Fr.zero),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, Fr.zero),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, Fr.zero),
      Fr.ZERO,
      makeTuple(NUM_FIELDS_PER_SHA256, Fr.zero),
      makeTuple(NUM_FIELDS_PER_SHA256, Fr.zero),
      Fr.ZERO,
      Fr.ZERO,
      BlockHeader.empty(),
      ContractDeploymentData.empty(),
      Fr.ZERO,
      Fr.ZERO,
    );
  }

  isEmpty() {
    // eslint-disable-next-line jsdoc/require-jsdoc
    const isEmptyArray = (arr: { isEmpty: (...args: any[]) => boolean }[]) => isArrayEmpty(arr, item => item.isEmpty());
    // eslint-disable-next-line jsdoc/require-jsdoc
    const isZeroArray = (arr: { isZero: (...args: any[]) => boolean }[]) => isArrayEmpty(arr, item => item.isZero());
    return (
      this.callContext.isEmpty() &&
      this.argsHash.isZero() &&
      isZeroArray(this.returnValues) &&
      isEmptyArray(this.readRequests) &&
      isEmptyArray(this.nullifierKeyValidationRequests) &&
      isEmptyArray(this.newCommitments) &&
      isEmptyArray(this.newNullifiers) &&
      isZeroArray(this.privateCallStackHashes) &&
      isZeroArray(this.publicCallStackHashes) &&
      isZeroArray(this.newL2ToL1Msgs) &&
      isZeroArray(this.encryptedLogsHash) &&
      isZeroArray(this.unencryptedLogsHash) &&
      this.encryptedLogPreimagesLength.isZero() &&
      this.unencryptedLogPreimagesLength.isZero() &&
      this.blockHeader.isEmpty() &&
      this.contractDeploymentData.isEmpty() &&
      this.chainId.isZero() &&
      this.version.isZero()
    );
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<PrivateCircuitPublicInputs>) {
    return [
      fields.callContext,
      fields.argsHash,
      fields.returnValues,
      fields.readRequests,
      fields.nullifierKeyValidationRequests,
      fields.newCommitments,
      fields.newNullifiers,
      fields.privateCallStackHashes,
      fields.publicCallStackHashes,
      fields.newL2ToL1Msgs,
      fields.endSideEffectCounter,
      fields.encryptedLogsHash,
      fields.unencryptedLogsHash,
      fields.encryptedLogPreimagesLength,
      fields.unencryptedLogPreimagesLength,
      fields.blockHeader,
      fields.contractDeploymentData,
      fields.chainId,
      fields.version,
    ] as const;
  }
  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(...PrivateCircuitPublicInputs.getFields(this));
  }
}

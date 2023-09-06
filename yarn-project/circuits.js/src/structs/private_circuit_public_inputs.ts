import { Fr } from '@aztec/foundation/fields';
import { Tuple } from '@aztec/foundation/serialize';

import {
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_READ_REQUESTS_PER_CALL,
  NUM_FIELDS_PER_SHA256,
  RETURN_VALUES_LENGTH,
} from '../cbind/constants.gen.js';
import { FieldsOf, assertMemberLength, makeTuple } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { CallContext } from './call_context.js';
import { HistoricBlockData } from './index.js';
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
    public readRequests: Tuple<Fr, typeof MAX_READ_REQUESTS_PER_CALL>,
    /**
     * New commitments created by the corresponding function call.
     */
    public newCommitments: Tuple<Fr, typeof MAX_NEW_COMMITMENTS_PER_CALL>,
    /**
     * New nullifiers created by the corresponding function call.
     */
    public newNullifiers: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_CALL>,
    /**
     * The commitments those were nullified by the above newNullifiers.
     */
    public nullifiedCommitments: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_CALL>,
    /**
     * Private call stack at the current kernel iteration.
     */
    public privateCallStack: Tuple<Fr, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    /**
     * Public call stack at the current kernel iteration.
     */
    public publicCallStack: Tuple<Fr, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    /**
     * New L2 to L1 messages created by the corresponding function call.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
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
     * Historic roots of the data trees, used to calculate the block hash the user is proving against.
     */
    public historicBlockData: HistoricBlockData,
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
  ) {
    assertMemberLength(this, 'returnValues', RETURN_VALUES_LENGTH);
    assertMemberLength(this, 'readRequests', MAX_READ_REQUESTS_PER_CALL);
    assertMemberLength(this, 'newCommitments', MAX_NEW_COMMITMENTS_PER_CALL);
    assertMemberLength(this, 'newNullifiers', MAX_NEW_NULLIFIERS_PER_CALL);
    assertMemberLength(this, 'nullifiedCommitments', MAX_NEW_NULLIFIERS_PER_CALL);
    assertMemberLength(this, 'privateCallStack', MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL);
    assertMemberLength(this, 'publicCallStack', MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
    assertMemberLength(this, 'newL2ToL1Msgs', MAX_NEW_L2_TO_L1_MSGS_PER_CALL);
    assertMemberLength(this, 'encryptedLogsHash', NUM_FIELDS_PER_SHA256);
    assertMemberLength(this, 'unencryptedLogsHash', NUM_FIELDS_PER_SHA256);
  }
  /**
   * Create PrivateCircuitPublicInputs from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A PrivateCircuitPublicInputs object.
   */
  static from(fields: FieldsOf<PrivateCircuitPublicInputs>): PrivateCircuitPublicInputs {
    return new PrivateCircuitPublicInputs(...PrivateCircuitPublicInputs.getFields(fields));
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
      makeTuple(MAX_READ_REQUESTS_PER_CALL, Fr.zero),
      makeTuple(MAX_NEW_COMMITMENTS_PER_CALL, Fr.zero),
      makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, Fr.zero),
      makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, Fr.zero),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, Fr.zero),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, Fr.zero),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, Fr.zero),
      makeTuple(NUM_FIELDS_PER_SHA256, Fr.zero),
      makeTuple(NUM_FIELDS_PER_SHA256, Fr.zero),
      Fr.ZERO,
      Fr.ZERO,
      HistoricBlockData.empty(),
      ContractDeploymentData.empty(),
      Fr.ZERO,
      Fr.ZERO,
    );
  }
  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<PrivateCircuitPublicInputs>) {
    return [
      // NOTE: Must have same order as CPP.
      fields.callContext,
      fields.argsHash,
      fields.returnValues,
      fields.readRequests,
      fields.newCommitments,
      fields.newNullifiers,
      fields.nullifiedCommitments,
      fields.privateCallStack,
      fields.publicCallStack,
      fields.newL2ToL1Msgs,
      fields.encryptedLogsHash,
      fields.unencryptedLogsHash,
      fields.encryptedLogPreimagesLength,
      fields.unencryptedLogPreimagesLength,
      fields.historicBlockData,
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

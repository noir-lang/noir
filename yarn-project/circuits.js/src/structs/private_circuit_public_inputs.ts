import { Fr } from '@aztec/foundation/fields';
import { assertMemberLength, FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { CallContext } from './call_context.js';
import {
  NEW_COMMITMENTS_LENGTH,
  NEW_L2_TO_L1_MSGS_LENGTH,
  NEW_NULLIFIERS_LENGTH,
  NUM_FIELDS_PER_SHA256,
  PRIVATE_CALL_STACK_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  READ_REQUESTS_LENGTH,
  RETURN_VALUES_LENGTH,
} from './constants.js';
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
    public returnValues: Fr[],
    /**
     * Read requests created by the corresponding function call.
     */
    public readRequests: Fr[],
    /**
     * New commitments created by the corresponding function call.
     */
    public newCommitments: Fr[],
    /**
     * New nullifiers created by the corresponding function call.
     */
    public newNullifiers: Fr[],
    /**
     * Private call stack at the current kernel iteration.
     */
    public privateCallStack: Fr[],
    /**
     * Public call stack at the current kernel iteration.
     */
    public publicCallStack: Fr[],
    /**
     * New L2 to L1 messages created by the corresponding function call.
     */
    public newL2ToL1Msgs: Fr[],
    /**
     * Hash of the encrypted logs emitted in this function call.
     * Note: Represented as an array of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public encryptedLogsHash: [Fr, Fr],
    /**
     * Hash of the unencrypted logs emitted in this function call.
     * Note: Represented as an array of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public unencryptedLogsHash: [Fr, Fr],
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
     * Root of the private data tree roots tree.
     */
    public historicPrivateDataTreeRoot: Fr,
    /**
     * Root of the nullifier tree roots tree.
     */
    public historicPrivateNullifierTreeRoot: Fr,
    /**
     * Root of the contract tree roots tree.
     */
    public historicContractTreeRoot: Fr,
    /**
     * Root of the L2 to L1 messages tree roots tree.
     */
    public historicL1ToL2MessagesTreeRoot: Fr,
    /**
     * Deployment data of contracts being deployed in this kernel iteration.
     */
    public contractDeploymentData: ContractDeploymentData,
  ) {
    assertMemberLength(this, 'returnValues', RETURN_VALUES_LENGTH);
    assertMemberLength(this, 'readRequests', READ_REQUESTS_LENGTH);
    assertMemberLength(this, 'newCommitments', NEW_COMMITMENTS_LENGTH);
    assertMemberLength(this, 'newNullifiers', NEW_NULLIFIERS_LENGTH);
    assertMemberLength(this, 'privateCallStack', PRIVATE_CALL_STACK_LENGTH);
    assertMemberLength(this, 'publicCallStack', PUBLIC_CALL_STACK_LENGTH);
    assertMemberLength(this, 'newL2ToL1Msgs', NEW_L2_TO_L1_MSGS_LENGTH);
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
    const frArray = (num: number) =>
      Array(num)
        .fill(0)
        .map(() => Fr.ZERO);
    return new PrivateCircuitPublicInputs(
      CallContext.empty(),
      Fr.ZERO,
      frArray(RETURN_VALUES_LENGTH),
      frArray(READ_REQUESTS_LENGTH),
      frArray(NEW_COMMITMENTS_LENGTH),
      frArray(NEW_NULLIFIERS_LENGTH),
      frArray(PRIVATE_CALL_STACK_LENGTH),
      frArray(PUBLIC_CALL_STACK_LENGTH),
      frArray(NEW_L2_TO_L1_MSGS_LENGTH),
      frArray(NUM_FIELDS_PER_SHA256) as [Fr, Fr],
      frArray(NUM_FIELDS_PER_SHA256) as [Fr, Fr],
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      ContractDeploymentData.empty(),
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
      fields.privateCallStack,
      fields.publicCallStack,
      fields.newL2ToL1Msgs,
      fields.encryptedLogsHash,
      fields.unencryptedLogsHash,
      fields.encryptedLogPreimagesLength,
      fields.unencryptedLogPreimagesLength,
      fields.historicPrivateDataTreeRoot,
      fields.historicPrivateNullifierTreeRoot,
      fields.historicContractTreeRoot,
      fields.historicL1ToL2MessagesTreeRoot,
      fields.contractDeploymentData,
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

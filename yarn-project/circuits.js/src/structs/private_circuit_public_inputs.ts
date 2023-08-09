import { Fr } from '@aztec/foundation/fields';

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
import { FieldsOf, assertMemberLength } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { CallContext } from './call_context.js';
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
     * The commitments those were nullified by the above newNullifiers.
     */
    public nullifiedCommitments: Fr[],
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
    public encryptedLogsHash: Fr[],
    /**
     * Hash of the unencrypted logs emitted in this function call.
     * Note: Represented as an array of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public unencryptedLogsHash: Fr[],
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
     * Root of the L2 to L1 messages tree.
     */
    public historicL1ToL2MessagesTreeRoot: Fr,
    /**
     * Root of the Blocks roots tree.
     */
    public historicBlocksTreeRoot: Fr,
    /**
     * Previous blocks global variables hash.
     */
    public historicGlobalVariablesHash: Fr,
    /**
     * Root of the Public Data tree.
     */
    public historicPublicDataTreeRoot: Fr,
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
    const frArray = (num: number) =>
      Array(num)
        .fill(0)
        .map(() => Fr.ZERO);
    return new PrivateCircuitPublicInputs(
      CallContext.empty(),
      Fr.ZERO,
      frArray(RETURN_VALUES_LENGTH),
      frArray(MAX_READ_REQUESTS_PER_CALL),
      frArray(MAX_NEW_COMMITMENTS_PER_CALL),
      frArray(MAX_NEW_NULLIFIERS_PER_CALL),
      frArray(MAX_NEW_NULLIFIERS_PER_CALL),
      frArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL),
      frArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL),
      frArray(MAX_NEW_L2_TO_L1_MSGS_PER_CALL),
      frArray(NUM_FIELDS_PER_SHA256),
      frArray(NUM_FIELDS_PER_SHA256),
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
      Fr.ZERO,
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
      fields.historicPrivateDataTreeRoot,
      fields.historicPrivateNullifierTreeRoot,
      fields.historicContractTreeRoot,
      fields.historicL1ToL2MessagesTreeRoot,
      fields.historicBlocksTreeRoot,
      fields.historicGlobalVariablesHash,
      fields.historicPublicDataTreeRoot,
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

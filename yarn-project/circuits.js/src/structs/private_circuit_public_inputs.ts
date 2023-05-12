import { Fr } from '@aztec/foundation/fields';
import { assertLength, FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { CallContext } from './call_context.js';
import {
  ARGS_LENGTH,
  EMITTED_EVENTS_LENGTH,
  NEW_L2_TO_L1_MSGS_LENGTH,
  NEW_COMMITMENTS_LENGTH,
  NEW_NULLIFIERS_LENGTH,
  PRIVATE_CALL_STACK_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  RETURN_VALUES_LENGTH,
} from './constants.js';
import { ContractDeploymentData } from './tx_context.js';

/**
 * Public inputs to a private circuit.
 * @see abis/private_circuit_public_inputs.hpp.
 */
export class PrivateCircuitPublicInputs {
  // NOTE: Args must have same order as CPP.
  constructor(
    /**
     * Context of the call corresponding to this private circuit execution.
     */
    public callContext: CallContext,
    /**
     * Function arguments.
     */
    public args: Fr[],
    /**
     * Return values of the corresponding function call.
     */
    public returnValues: Fr[],
    /**
     * Events emitted by the corresponding function call.
     */
    public emittedEvents: Fr[],
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
    assertLength(this, 'args', ARGS_LENGTH);
    assertLength(this, 'returnValues', RETURN_VALUES_LENGTH);
    assertLength(this, 'emittedEvents', EMITTED_EVENTS_LENGTH);
    assertLength(this, 'newCommitments', NEW_COMMITMENTS_LENGTH);
    assertLength(this, 'newNullifiers', NEW_NULLIFIERS_LENGTH);
    assertLength(this, 'privateCallStack', PRIVATE_CALL_STACK_LENGTH);
    assertLength(this, 'publicCallStack', PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, 'newL2ToL1Msgs', NEW_L2_TO_L1_MSGS_LENGTH);
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
      frArray(ARGS_LENGTH),
      frArray(RETURN_VALUES_LENGTH),
      frArray(EMITTED_EVENTS_LENGTH),
      frArray(NEW_COMMITMENTS_LENGTH),
      frArray(NEW_NULLIFIERS_LENGTH),
      frArray(PRIVATE_CALL_STACK_LENGTH),
      frArray(PUBLIC_CALL_STACK_LENGTH),
      frArray(NEW_L2_TO_L1_MSGS_LENGTH),
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
      fields.args,
      fields.returnValues,
      fields.emittedEvents,
      fields.newCommitments,
      fields.newNullifiers,
      fields.privateCallStack,
      fields.publicCallStack,
      fields.newL2ToL1Msgs,
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

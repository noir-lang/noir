import { Fr } from '@aztec/foundation';
import { assertLength, FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { CallContext } from './call_context.js';
import {
  ARGS_LENGTH,
  EMITTED_EVENTS_LENGTH,
  L1_MSG_STACK_LENGTH,
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
  constructor(
    // NOTE: Must have same order as CPP.
    public callContext: CallContext,
    public args: Fr[],
    public returnValues: Fr[],
    public emittedEvents: Fr[],
    public newCommitments: Fr[],
    public newNullifiers: Fr[],
    public privateCallStack: Fr[],
    public publicCallStack: Fr[],
    public l1MsgStack: Fr[],
    public historicPrivateDataTreeRoot: Fr,
    public historicPrivateNullifierTreeRoot: Fr,
    public historicContractTreeRoot: Fr,
    public contractDeploymentData: ContractDeploymentData,
  ) {
    assertLength(this, 'args', ARGS_LENGTH);
    assertLength(this, 'returnValues', RETURN_VALUES_LENGTH);
    assertLength(this, 'emittedEvents', EMITTED_EVENTS_LENGTH);
    assertLength(this, 'newCommitments', NEW_COMMITMENTS_LENGTH);
    assertLength(this, 'newNullifiers', NEW_NULLIFIERS_LENGTH);
    assertLength(this, 'privateCallStack', PRIVATE_CALL_STACK_LENGTH);
    assertLength(this, 'publicCallStack', PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, 'l1MsgStack', L1_MSG_STACK_LENGTH);
  }
  /**
   * Create PrivateCircuitPublicInputs from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A PrivateCircuitPublicInputs object.
   */
  static from(fields: FieldsOf<PrivateCircuitPublicInputs>): PrivateCircuitPublicInputs {
    return new PrivateCircuitPublicInputs(...PrivateCircuitPublicInputs.getFields(fields));
  }

  public static empty() {
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
      frArray(L1_MSG_STACK_LENGTH),
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
      fields.l1MsgStack,
      fields.historicPrivateDataTreeRoot,
      fields.historicPrivateNullifierTreeRoot,
      fields.historicContractTreeRoot,
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

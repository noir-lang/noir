import { AztecAddress, Fr } from '@aztec/foundation';
import times from 'lodash.times';
import { FieldsOf, assertLength } from '../utils/jsUtils.js';
import { CallContext } from './call_context.js';
import {
  ARGS_LENGTH,
  EMITTED_EVENTS_LENGTH,
  L1_MSG_STACK_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  RETURN_VALUES_LENGTH,
  STATE_READS_LENGTH,
  STATE_TRANSITIONS_LENGTH,
} from './constants.js';

/**
 * Read operations from the public state tree.
 */
export interface StateRead {
  storageSlot: Fr;
  value: Fr;
}

/**
 * Write operations on the public state tree.
 */
export interface StateTransition {
  storageSlot: Fr;
  oldValue: Fr;
  newValue: Fr;
}

/**
 * Public inputs to a public circuit.
 */
export class PublicCircuitPublicInputs {
  constructor(
    public callContext: CallContext,
    public args: Fr[],
    public returnValues: Fr[],
    public emittedEvents: Fr[],
    public publicCallStack: Fr[],
    public l1MsgStack: Fr[],
    public stateTransitions: StateTransition[],
    public stateReads: StateRead[],
    public proverAddress: AztecAddress,
  ) {
    assertLength(this, 'args', ARGS_LENGTH);
    assertLength(this, 'returnValues', RETURN_VALUES_LENGTH);
    assertLength(this, 'emittedEvents', EMITTED_EVENTS_LENGTH);
    assertLength(this, 'publicCallStack', PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, 'l1MsgStack', L1_MSG_STACK_LENGTH);
    assertLength(this, 'stateTransitions', STATE_TRANSITIONS_LENGTH);
    assertLength(this, 'stateReads', STATE_READS_LENGTH);
  }
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
   * @returns an empty instance.
   */
  public static empty() {
    const frArray = (num: number) => times(num, () => Fr.ZERO);
    return new PublicCircuitPublicInputs(
      CallContext.empty(),
      frArray(ARGS_LENGTH),
      frArray(RETURN_VALUES_LENGTH),
      frArray(EMITTED_EVENTS_LENGTH),
      frArray(PUBLIC_CALL_STACK_LENGTH),
      frArray(L1_MSG_STACK_LENGTH),
      times(STATE_TRANSITIONS_LENGTH, () => ({ storageSlot: Fr.ZERO, oldValue: Fr.ZERO, newValue: Fr.ZERO })),
      times(STATE_READS_LENGTH, () => ({ storageSlot: Fr.ZERO, value: Fr.ZERO })),
      AztecAddress.ZERO,
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
      fields.args,
      fields.returnValues,
      fields.emittedEvents,
      fields.publicCallStack,
      fields.l1MsgStack,
      fields.stateTransitions,
      fields.stateReads,
      fields.proverAddress,
    ] as const;
  }
}

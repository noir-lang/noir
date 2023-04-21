import { AztecAddress, BufferReader, Fr } from '@aztec/foundation';
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
import { serializeToBuffer } from '../utils/serialize.js';

/**
 * Public state read operation on a specific contract.
 */
export class StateRead {
  constructor(public readonly storageSlot: Fr, public readonly value: Fr) {}

  static from(args: { storageSlot: Fr; value: Fr }) {
    return new StateRead(args.storageSlot, args.value);
  }

  toBuffer() {
    return serializeToBuffer(this.storageSlot, this.value);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new StateRead(reader.readFr(), reader.readFr());
  }

  static empty() {
    return new StateRead(Fr.ZERO, Fr.ZERO);
  }
}

/**
 * Public state transition for a slot on a specific contract.
 */
export class StateTransition {
  constructor(public readonly storageSlot: Fr, public readonly oldValue: Fr, public readonly newValue: Fr) {}

  static from(args: { storageSlot: Fr; oldValue: Fr; newValue: Fr }) {
    return new StateTransition(args.storageSlot, args.oldValue, args.newValue);
  }

  toBuffer() {
    return serializeToBuffer(this.storageSlot, this.oldValue, this.newValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new StateTransition(reader.readFr(), reader.readFr(), reader.readFr());
  }

  static empty() {
    return new StateTransition(Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }
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
    public stateTransitions: StateTransition[],
    public stateReads: StateRead[],
    public publicCallStack: Fr[],
    public l1MsgStack: Fr[],
    public historicPublicDataTreeRoot: Fr,
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
      times(STATE_TRANSITIONS_LENGTH, StateTransition.empty),
      times(STATE_READS_LENGTH, StateRead.empty),
      frArray(PUBLIC_CALL_STACK_LENGTH),
      frArray(L1_MSG_STACK_LENGTH),
      Fr.ZERO,
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
      fields.stateTransitions,
      fields.stateReads,
      fields.publicCallStack,
      fields.l1MsgStack,
      fields.historicPublicDataTreeRoot,
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
}

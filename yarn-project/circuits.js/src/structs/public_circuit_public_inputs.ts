import times from 'lodash.times';
import { FieldsOf, assertLength } from '../utils/jsUtils.js';
import { CallContext } from './call_context.js';
import {
  ARGS_LENGTH,
  EMITTED_EVENTS_LENGTH,
  NEW_L2_TO_L1_MSGS_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  RETURN_VALUES_LENGTH,
  KERNEL_PUBLIC_DATA_READS_LENGTH,
  KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
} from './constants.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { isArrayEmpty } from '@aztec/foundation/collection';

/**
 * Contract storage read operation on a specific contract.
 *
 * Note: Similar to `PublicDataRead` but it's from the POV of contract storage so we are not working with public data
 * tree leaf index but storage slot index.
 */
export class ContractStorageRead {
  constructor(
    /**
     * Storage slot we are reading from.
     */
    public readonly storageSlot: Fr,
    /**
     * Value read from the storage slot.
     */
    public readonly value: Fr,
  ) {}

  static from(args: {
    /**
     * Storage slot we are reading from.
     */
    storageSlot: Fr;
    /**
     * Value read from the storage slot.
     */
    value: Fr;
  }) {
    return new ContractStorageRead(args.storageSlot, args.value);
  }

  toBuffer() {
    return serializeToBuffer(this.storageSlot, this.value);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ContractStorageRead(reader.readFr(), reader.readFr());
  }

  static empty() {
    return new ContractStorageRead(Fr.ZERO, Fr.ZERO);
  }

  isEmpty() {
    return this.storageSlot.isZero() && this.value.isZero();
  }

  toFriendlyJSON() {
    return `Slot=${this.storageSlot.toFriendlyJSON()}: ${this.value.toFriendlyJSON()}`;
  }
}

/**
 * Contract storage update request for a slot on a specific contract.
 *
 * Note: Similar to `PublicDataUpdateRequest` but it's from the POV of contract storage so we are not working with
 * public data tree leaf index but storage slot index.
 */
export class ContractStorageUpdateRequest {
  constructor(
    /**
     * Storage slot we are updating.
     */
    public readonly storageSlot: Fr,
    /**
     * Old value of the storage slot.
     */
    public readonly oldValue: Fr,
    /**
     * New value of the storage slot.
     */
    public readonly newValue: Fr,
  ) {}

  static from(args: {
    /**
     * Storage slot we are updating.
     */
    storageSlot: Fr;
    /**
     * Old value of the storage slot.
     */
    oldValue: Fr;
    /**
     * New value of the storage slot.
     */
    newValue: Fr;
  }) {
    return new ContractStorageUpdateRequest(args.storageSlot, args.oldValue, args.newValue);
  }

  toBuffer() {
    return serializeToBuffer(this.storageSlot, this.oldValue, this.newValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ContractStorageUpdateRequest(reader.readFr(), reader.readFr(), reader.readFr());
  }

  static empty() {
    return new ContractStorageUpdateRequest(Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }

  isEmpty() {
    return this.storageSlot.isZero() && this.oldValue.isZero() && this.newValue.isZero();
  }

  toFriendlyJSON() {
    return `Slot=${this.storageSlot.toFriendlyJSON()}: ${this.oldValue.toFriendlyJSON()} => ${this.newValue.toFriendlyJSON()}`;
  }
}

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
     * Arguments of the call.
     */
    public args: Fr[],
    /**
     * Return values of the call.
     */
    public returnValues: Fr[],
    /**
     * Events emitted during the call.
     */
    public emittedEvents: Fr[],
    /**
     * Contract storage update requests executed during the call.
     */
    public contractStorageUpdateRequests: ContractStorageUpdateRequest[],
    /**
     * Contract storage reads executed during the call.
     */
    public contractStorageReads: ContractStorageRead[],
    /**
     * Public call stack of the current kernel iteration.
     */
    public publicCallStack: Fr[],
    /**
     * New L2 to L1 messages generated during the call.
     */
    public newL2ToL1Msgs: Fr[],
    /**
     * Root of the public data tree when the call started.
     */
    public historicPublicDataTreeRoot: Fr,
    /**
     * Address of the prover.
     */
    public proverAddress: AztecAddress,
  ) {
    assertLength(this, 'args', ARGS_LENGTH);
    assertLength(this, 'returnValues', RETURN_VALUES_LENGTH);
    assertLength(this, 'emittedEvents', EMITTED_EVENTS_LENGTH);
    assertLength(this, 'publicCallStack', PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, 'newL2ToL1Msgs', NEW_L2_TO_L1_MSGS_LENGTH);
    assertLength(this, 'contractStorageUpdateRequests', KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH);
    assertLength(this, 'contractStorageReads', KERNEL_PUBLIC_DATA_READS_LENGTH);
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
   * @returns An empty instance.
   */
  public static empty() {
    const frArray = (num: number) => times(num, () => Fr.ZERO);
    return new PublicCircuitPublicInputs(
      CallContext.empty(),
      frArray(ARGS_LENGTH),
      frArray(RETURN_VALUES_LENGTH),
      frArray(EMITTED_EVENTS_LENGTH),
      times(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH, ContractStorageUpdateRequest.empty),
      times(KERNEL_PUBLIC_DATA_READS_LENGTH, ContractStorageRead.empty),
      frArray(PUBLIC_CALL_STACK_LENGTH),
      frArray(NEW_L2_TO_L1_MSGS_LENGTH),
      Fr.ZERO,
      AztecAddress.ZERO,
    );
  }

  isEmpty() {
    const isFrArrayEmpty = (arr: Fr[]) => isArrayEmpty(arr, item => item.isZero());
    return (
      this.callContext.isEmpty() &&
      isFrArrayEmpty(this.args) &&
      isFrArrayEmpty(this.returnValues) &&
      isFrArrayEmpty(this.emittedEvents) &&
      isArrayEmpty(this.contractStorageUpdateRequests, item => item.isEmpty()) &&
      isArrayEmpty(this.contractStorageReads, item => item.isEmpty()) &&
      isFrArrayEmpty(this.publicCallStack) &&
      isFrArrayEmpty(this.newL2ToL1Msgs) &&
      this.historicPublicDataTreeRoot.isZero() &&
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
      fields.args,
      fields.returnValues,
      fields.emittedEvents,
      fields.contractStorageUpdateRequests,
      fields.contractStorageReads,
      fields.publicCallStack,
      fields.newL2ToL1Msgs,
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

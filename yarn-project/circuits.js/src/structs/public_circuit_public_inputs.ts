import { FieldsOf, assertMemberLength, makeTuple } from '../utils/jsUtils.js';
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
import { BufferReader, Tuple } from '@aztec/foundation/serialize';
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
    public readonly currentValue: Fr,
  ) {}

  static from(args: {
    /**
     * Storage slot we are reading from.
     */
    storageSlot: Fr;
    /**
     * Value read from the storage slot.
     */
    currentValue: Fr;
  }) {
    return new ContractStorageRead(args.storageSlot, args.currentValue);
  }

  toBuffer() {
    return serializeToBuffer(this.storageSlot, this.currentValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ContractStorageRead(reader.readFr(), reader.readFr());
  }

  static empty() {
    return new ContractStorageRead(Fr.ZERO, Fr.ZERO);
  }

  isEmpty() {
    return this.storageSlot.isZero() && this.currentValue.isZero();
  }

  toFriendlyJSON() {
    return `Slot=${this.storageSlot.toFriendlyJSON()}: ${this.currentValue.toFriendlyJSON()}`;
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
    public args: Tuple<Fr, typeof ARGS_LENGTH>,
    /**
     * Return values of the call.
     */
    public returnValues: Tuple<Fr, typeof RETURN_VALUES_LENGTH>,
    /**
     * Events emitted during the call.
     */
    public emittedEvents: Tuple<Fr, typeof EMITTED_EVENTS_LENGTH>,
    /**
     * Contract storage update requests executed during the call.
     */
    public contractStorageUpdateRequests: Tuple<
      ContractStorageUpdateRequest,
      typeof KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH
    >,
    /**
     * Contract storage reads executed during the call.
     */
    public contractStorageReads: Tuple<ContractStorageRead, typeof KERNEL_PUBLIC_DATA_READS_LENGTH>,
    /**
     * Public call stack of the current kernel iteration.
     */
    public publicCallStack: Tuple<Fr, typeof PUBLIC_CALL_STACK_LENGTH>,
    /**
     * New L2 to L1 messages generated during the call.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof NEW_L2_TO_L1_MSGS_LENGTH>,
    /**
     * Root of the public data tree when the call started.
     */
    public historicPublicDataTreeRoot: Fr,
    /**
     * Address of the prover.
     */
    public proverAddress: AztecAddress,
  ) {
    assertMemberLength(this, 'args', ARGS_LENGTH);
    assertMemberLength(this, 'returnValues', RETURN_VALUES_LENGTH);
    assertMemberLength(this, 'emittedEvents', EMITTED_EVENTS_LENGTH);
    assertMemberLength(this, 'publicCallStack', PUBLIC_CALL_STACK_LENGTH);
    assertMemberLength(this, 'newL2ToL1Msgs', NEW_L2_TO_L1_MSGS_LENGTH);
    assertMemberLength(this, 'contractStorageUpdateRequests', KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH);
    assertMemberLength(this, 'contractStorageReads', KERNEL_PUBLIC_DATA_READS_LENGTH);
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
    return new PublicCircuitPublicInputs(
      CallContext.empty(),
      makeTuple(ARGS_LENGTH, Fr.zero),
      makeTuple(RETURN_VALUES_LENGTH, Fr.zero),
      makeTuple(EMITTED_EVENTS_LENGTH, Fr.zero),
      makeTuple(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH, ContractStorageUpdateRequest.empty),
      makeTuple(KERNEL_PUBLIC_DATA_READS_LENGTH, ContractStorageRead.empty),
      makeTuple(PUBLIC_CALL_STACK_LENGTH, Fr.zero),
      makeTuple(NEW_L2_TO_L1_MSGS_LENGTH, Fr.zero),
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

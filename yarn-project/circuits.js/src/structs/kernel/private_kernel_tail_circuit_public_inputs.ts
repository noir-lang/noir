import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { arraySerializedSizeOfNonEmpty } from '@aztec/foundation/collection';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX } from '../../constants.gen.js';
import { countAccumulatedItems, mergeAccumulatedData } from '../../utils/index.js';
import { CallRequest } from '../call_request.js';
import { PartialStateReference } from '../partial_state_reference.js';
import { RevertCode } from '../revert_code.js';
import { RollupValidationRequests } from '../rollup_validation_requests.js';
import { ValidationRequests } from '../validation_requests.js';
import { CombinedAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';
import { KernelCircuitPublicInputs } from './kernel_circuit_public_inputs.js';
import { PublicAccumulatedData } from './public_accumulated_data.js';
import { PublicKernelCircuitPublicInputs } from './public_kernel_circuit_public_inputs.js';

export class PartialPrivateTailPublicInputsForPublic {
  constructor(
    /**
     * Validation requests accumulated from public functions.
     */
    public validationRequests: ValidationRequests,
    /**
     * Accumulated side effects and enqueued calls that are not revertible.
     */
    public endNonRevertibleData: PublicAccumulatedData,
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: PublicAccumulatedData,
    /**
     * Call request for the public teardown function.
     */
    public publicTeardownCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
  ) {}

  getSize() {
    return (
      this.validationRequests.getSize() +
      this.endNonRevertibleData.getSize() +
      this.end.getSize() +
      arraySerializedSizeOfNonEmpty(this.publicTeardownCallStack)
    );
  }

  get needsSetup() {
    return !this.endNonRevertibleData.publicCallStack[0].isEmpty();
  }

  get needsAppLogic() {
    return !this.end.publicCallStack[0].isEmpty();
  }

  get needsTeardown() {
    return !this.publicTeardownCallStack[0].isEmpty();
  }

  static fromBuffer(buffer: Buffer | BufferReader): PartialPrivateTailPublicInputsForPublic {
    const reader = BufferReader.asReader(buffer);
    return new PartialPrivateTailPublicInputsForPublic(
      reader.readObject(ValidationRequests),
      reader.readObject(PublicAccumulatedData),
      reader.readObject(PublicAccumulatedData),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
    );
  }

  toBuffer() {
    return serializeToBuffer(
      this.validationRequests,
      this.endNonRevertibleData,
      this.end,
      this.publicTeardownCallStack,
    );
  }

  static empty() {
    return new PartialPrivateTailPublicInputsForPublic(
      ValidationRequests.empty(),
      PublicAccumulatedData.empty(),
      PublicAccumulatedData.empty(),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
    );
  }
}

export class PartialPrivateTailPublicInputsForRollup {
  constructor(public rollupValidationRequests: RollupValidationRequests, public end: CombinedAccumulatedData) {}

  static fromBuffer(buffer: Buffer | BufferReader): PartialPrivateTailPublicInputsForRollup {
    const reader = BufferReader.asReader(buffer);
    return new PartialPrivateTailPublicInputsForRollup(
      reader.readObject(RollupValidationRequests),
      reader.readObject(CombinedAccumulatedData),
    );
  }

  getSize() {
    return this.rollupValidationRequests.getSize() + this.end.getSize();
  }

  toBuffer() {
    return serializeToBuffer(this.rollupValidationRequests, this.end);
  }

  static empty() {
    return new PartialPrivateTailPublicInputsForRollup(
      RollupValidationRequests.empty(),
      CombinedAccumulatedData.empty(),
    );
  }
}

export class PrivateKernelTailCircuitPublicInputs {
  constructor(
    /**
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
    /**
     * Indicates whether execution of the public circuit reverted.
     */
    public revertCode: RevertCode,
    /**
     * The address of the fee payer for the transaction.
     */
    public feePayer: AztecAddress,

    public forPublic?: PartialPrivateTailPublicInputsForPublic,
    public forRollup?: PartialPrivateTailPublicInputsForRollup,
  ) {
    if (!forPublic && !forRollup) {
      throw new Error('Missing partial public inputs for private tail circuit.');
    }
    if (forPublic && forRollup) {
      throw new Error(
        'Cannot create PrivateKernelTailCircuitPublicInputs that is for both public kernel circuit and rollup circuit.',
      );
    }
  }

  get publicInputs(): PartialPrivateTailPublicInputsForPublic | PartialPrivateTailPublicInputsForRollup {
    return (this.forPublic ?? this.forRollup)!;
  }

  getSize() {
    return (
      (this.forPublic?.getSize() ?? 0) +
      (this.forRollup?.getSize() ?? 0) +
      this.constants.getSize() +
      this.revertCode.getSerializedLength() +
      this.feePayer.size
    );
  }

  toPublicKernelCircuitPublicInputs() {
    if (!this.forPublic) {
      throw new Error('Private tail public inputs is not for public circuit.');
    }
    return new PublicKernelCircuitPublicInputs(
      this.forPublic.validationRequests,
      this.forPublic.endNonRevertibleData,
      this.forPublic.end,
      this.constants,
      this.revertCode,
      this.forPublic.publicTeardownCallStack,
      this.feePayer,
    );
  }

  toKernelCircuitPublicInputs() {
    if (!this.forRollup) {
      throw new Error('Private tail public inputs is not for rollup circuit.');
    }
    return new KernelCircuitPublicInputs(
      this.forRollup.rollupValidationRequests,
      this.forRollup.end,
      this.constants,
      PartialStateReference.empty(),
      this.revertCode,
      this.feePayer,
    );
  }

  numberOfPublicCallRequests() {
    return this.forPublic
      ? countAccumulatedItems(this.forPublic.endNonRevertibleData.publicCallStack) +
          countAccumulatedItems(this.forPublic.end.publicCallStack) +
          countAccumulatedItems(this.forPublic.publicTeardownCallStack)
      : 0;
  }

  getNonEmptyNoteHashes() {
    const noteHashes = this.forPublic
      ? mergeAccumulatedData(this.forPublic.endNonRevertibleData.noteHashes, this.forPublic.end.noteHashes).map(
          n => n.value,
        )
      : this.forRollup!.end.noteHashes;
    return noteHashes.filter(n => !n.isZero());
  }

  getNonEmptyNullifiers() {
    const nullifiers = this.forPublic
      ? mergeAccumulatedData(this.forPublic.endNonRevertibleData.nullifiers, this.forPublic.end.nullifiers).map(
          n => n.value,
        )
      : this.forRollup!.end.nullifiers;
    return nullifiers.filter(n => !n.isZero());
  }

  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelTailCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    const isForPublic = reader.readBoolean();
    return new PrivateKernelTailCircuitPublicInputs(
      reader.readObject(CombinedConstantData),
      reader.readObject(RevertCode),
      reader.readObject(AztecAddress),
      isForPublic ? reader.readObject(PartialPrivateTailPublicInputsForPublic) : undefined,
      !isForPublic ? reader.readObject(PartialPrivateTailPublicInputsForRollup) : undefined,
    );
  }

  toBuffer() {
    const isForPublic = !!this.forPublic;
    return serializeToBuffer(
      isForPublic,
      this.constants,
      this.revertCode,
      this.feePayer,
      isForPublic ? this.forPublic!.toBuffer() : this.forRollup!.toBuffer(),
    );
  }

  static empty() {
    return new PrivateKernelTailCircuitPublicInputs(
      CombinedConstantData.empty(),
      RevertCode.OK,
      AztecAddress.ZERO,
      undefined,
      PartialPrivateTailPublicInputsForRollup.empty(),
    );
  }
}

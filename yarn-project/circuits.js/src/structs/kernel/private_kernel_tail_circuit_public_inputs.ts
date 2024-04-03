import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { MAX_NEW_NULLIFIERS_PER_TX } from '../../constants.gen.js';
import { countAccumulatedItems, mergeAccumulatedData } from '../../utils/index.js';
import { AggregationObject } from '../aggregation_object.js';
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
  ) {}

  get needsSetup() {
    return !this.endNonRevertibleData.publicCallStack[1].isEmpty();
  }

  get needsAppLogic() {
    return !this.end.publicCallStack[0].isEmpty();
  }

  get needsTeardown() {
    return !this.endNonRevertibleData.publicCallStack[0].isEmpty();
  }

  static fromBuffer(buffer: Buffer | BufferReader): PartialPrivateTailPublicInputsForPublic {
    const reader = BufferReader.asReader(buffer);
    return new PartialPrivateTailPublicInputsForPublic(
      reader.readObject(ValidationRequests),
      reader.readObject(PublicAccumulatedData),
      reader.readObject(PublicAccumulatedData),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.validationRequests, this.endNonRevertibleData, this.end);
  }

  static empty() {
    return new PartialPrivateTailPublicInputsForPublic(
      ValidationRequests.empty(),
      PublicAccumulatedData.empty(),
      PublicAccumulatedData.empty(),
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
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
    /**
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
    /**
     * Indicates whether execution of the public circuit reverted.
     */
    public revertCode: RevertCode,
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

  toPublicKernelCircuitPublicInputs() {
    if (!this.forPublic) {
      throw new Error('Private tail public inputs is not for public circuit.');
    }
    return new PublicKernelCircuitPublicInputs(
      this.aggregationObject,
      this.forPublic.validationRequests,
      this.forPublic.endNonRevertibleData,
      this.forPublic.end,
      this.constants,
      this.revertCode,
    );
  }

  toKernelCircuitPublicInputs() {
    if (!this.forRollup) {
      throw new Error('Private tail public inputs is not for rollup circuit.');
    }
    return new KernelCircuitPublicInputs(
      this.aggregationObject,
      this.forRollup.rollupValidationRequests,
      this.forRollup.end,
      this.constants,
      this.revertCode,
    );
  }

  numberOfPublicCallRequests() {
    return this.forPublic
      ? countAccumulatedItems(this.forPublic.endNonRevertibleData.publicCallStack) +
          countAccumulatedItems(this.forPublic.end.publicCallStack)
      : 0;
  }

  getNonEmptyNoteHashes() {
    const noteHashes = this.forPublic
      ? mergeAccumulatedData(
          MAX_NEW_NULLIFIERS_PER_TX,
          this.forPublic.endNonRevertibleData.newNoteHashes,
          this.forPublic.end.newNoteHashes,
        )
      : this.forRollup!.end.newNoteHashes;
    return noteHashes.filter(n => !n.isEmpty());
  }

  getNonEmptyNullifiers() {
    const nullifiers = this.forPublic
      ? mergeAccumulatedData(
          MAX_NEW_NULLIFIERS_PER_TX,
          this.forPublic.endNonRevertibleData.newNullifiers,
          this.forPublic.end.newNullifiers,
        )
      : this.forRollup!.end.newNullifiers;
    return nullifiers.filter(n => !n.isEmpty());
  }

  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelTailCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    const isForPublic = reader.readBoolean();
    return new PrivateKernelTailCircuitPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(CombinedConstantData),
      reader.readObject(RevertCode),
      isForPublic ? reader.readObject(PartialPrivateTailPublicInputsForPublic) : undefined,
      !isForPublic ? reader.readObject(PartialPrivateTailPublicInputsForRollup) : undefined,
    );
  }

  toBuffer() {
    const isForPublic = !!this.forPublic;
    return serializeToBuffer(
      isForPublic,
      this.aggregationObject,
      this.constants,
      this.revertCode,
      isForPublic ? this.forPublic!.toBuffer() : this.forRollup!.toBuffer(),
    );
  }

  static empty() {
    return new PrivateKernelTailCircuitPublicInputs(
      AggregationObject.makeFake(),
      CombinedConstantData.empty(),
      RevertCode.OK,
      undefined,
      PartialPrivateTailPublicInputsForRollup.empty(),
    );
  }
}

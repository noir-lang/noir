import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX } from '../../constants.gen.js';
import { countAccumulatedItems, mergeAccumulatedData } from '../../utils/index.js';
import { PartialStateReference } from '../partial_state_reference.js';
import { PublicCallRequest } from '../public_call_request.js';
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
    public publicTeardownCallRequest: PublicCallRequest,
  ) {}

  getSize() {
    return (
      this.validationRequests.getSize() +
      this.endNonRevertibleData.getSize() +
      this.end.getSize() +
      this.publicTeardownCallRequest.getSize()
    );
  }

  get needsSetup() {
    return !this.endNonRevertibleData.publicCallStack[0].isEmpty();
  }

  get needsAppLogic() {
    return !this.end.publicCallStack[0].isEmpty();
  }

  get needsTeardown() {
    return !this.publicTeardownCallRequest.isEmpty();
  }

  static fromBuffer(buffer: Buffer | BufferReader): PartialPrivateTailPublicInputsForPublic {
    const reader = BufferReader.asReader(buffer);
    return new PartialPrivateTailPublicInputsForPublic(
      reader.readObject(ValidationRequests),
      reader.readObject(PublicAccumulatedData),
      reader.readObject(PublicAccumulatedData),
      reader.readObject(PublicCallRequest),
    );
  }

  toBuffer() {
    return serializeToBuffer(
      this.validationRequests,
      this.endNonRevertibleData,
      this.end,
      this.publicTeardownCallRequest,
    );
  }

  static empty() {
    return new PartialPrivateTailPublicInputsForPublic(
      ValidationRequests.empty(),
      PublicAccumulatedData.empty(),
      PublicAccumulatedData.empty(),
      PublicCallRequest.empty(),
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
      padArrayEnd(
        [this.forPublic.publicTeardownCallRequest],
        PublicCallRequest.empty(),
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      ),
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
    return (
      this.numberOfNonRevertiblePublicCallRequests() +
      this.numberOfRevertiblePublicCallRequests() +
      (this.hasTeardownPublicCallRequest() ? 1 : 0)
    );
  }

  numberOfNonRevertiblePublicCallRequests() {
    return this.forPublic ? countAccumulatedItems(this.forPublic.endNonRevertibleData.publicCallStack) : 0;
  }

  numberOfRevertiblePublicCallRequests() {
    return this.forPublic ? countAccumulatedItems(this.forPublic.end.publicCallStack) : 0;
  }

  hasTeardownPublicCallRequest() {
    return this.forPublic ? !this.forPublic.publicTeardownCallRequest.isEmpty() : false;
  }

  getNonRevertiblePublicCallRequests() {
    return this.forPublic ? this.forPublic.endNonRevertibleData.publicCallStack.filter(r => !r.isEmpty()) : [];
  }

  getRevertiblePublicCallRequests() {
    return this.forPublic ? this.forPublic.end.publicCallStack.filter(r => !r.isEmpty()) : [];
  }

  getTeardownPublicCallRequest() {
    const publicTeardownCallRequest = this.forPublic?.publicTeardownCallRequest;
    return !publicTeardownCallRequest?.isEmpty() ? publicTeardownCallRequest : undefined;
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

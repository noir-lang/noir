import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import { MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX } from '../../constants.gen.js';
import { PublicCallRequest } from '../public_call_request.js';
import { RevertCode } from '../revert_code.js';
import { ValidationRequests } from '../validation_requests.js';
import { CombinedConstantData } from './combined_constant_data.js';
import { PublicAccumulatedData } from './public_accumulated_data.js';

/**
 * Outputs from the public kernel circuits.
 * All Public kernels use this shape for outputs.
 */
export class PublicKernelCircuitPublicInputs {
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
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
    /**
     * Indicates whether execution of the public circuit reverted.
     */
    public revertCode: RevertCode,
    /**
     * The call request for the public teardown function
     */
    public publicTeardownCallStack: Tuple<PublicCallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    /**
     * The address of the fee payer for the transaction
     */
    public feePayer: AztecAddress,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.validationRequests,
      this.endNonRevertibleData,
      this.end,
      this.constants,
      this.revertCode,
      this.publicTeardownCallStack,
      this.feePayer,
    );
  }

  clone() {
    return PublicKernelCircuitPublicInputs.fromBuffer(this.toBuffer());
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static fromString(str: string) {
    return PublicKernelCircuitPublicInputs.fromBuffer(Buffer.from(str, 'hex'));
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

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of PublicKernelCircuitPublicInputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PublicKernelCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new PublicKernelCircuitPublicInputs(
      reader.readObject(ValidationRequests),
      reader.readObject(PublicAccumulatedData),
      reader.readObject(PublicAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readObject(RevertCode),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, PublicCallRequest),
      reader.readObject(AztecAddress),
    );
  }

  static empty() {
    return new PublicKernelCircuitPublicInputs(
      ValidationRequests.empty(),
      PublicAccumulatedData.empty(),
      PublicAccumulatedData.empty(),
      CombinedConstantData.empty(),
      RevertCode.OK,
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, PublicCallRequest.empty),
      AztecAddress.ZERO,
    );
  }

  static fromFields(fields: Fr[] | FieldReader): PublicKernelCircuitPublicInputs {
    const reader = FieldReader.asReader(fields);
    return new PublicKernelCircuitPublicInputs(
      ValidationRequests.fromFields(reader),
      PublicAccumulatedData.fromFields(reader),
      PublicAccumulatedData.fromFields(reader),
      CombinedConstantData.fromFields(reader),
      RevertCode.fromField(reader.readField()),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, PublicCallRequest),
      AztecAddress.fromFields(reader),
    );
  }

  [inspect.custom]() {
    return `PublicKernelCircuitPublicInputs {
      validationRequests: ${inspect(this.validationRequests)},
      endNonRevertibleData: ${inspect(this.endNonRevertibleData)},
      end: ${inspect(this.end)},
      constants: ${inspect(this.constants)},
      revertCode: ${this.revertCode},
      publicTeardownCallStack: ${inspect(this.publicTeardownCallStack)}
      feePayer: ${this.feePayer}
      }`;
  }
}

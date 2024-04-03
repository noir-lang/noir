import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import { AggregationObject } from '../aggregation_object.js';
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
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
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
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.aggregationObject,
      this.validationRequests,
      this.endNonRevertibleData,
      this.end,
      this.constants,
      this.revertCode,
    );
  }

  get needsSetup() {
    return !this.endNonRevertibleData.publicCallStack[1].isEmpty();
  }

  get needsAppLogic() {
    return !this.end.publicCallStack[0].isEmpty();
  }

  get needsTeardown() {
    return !this.endNonRevertibleData.publicCallStack[0].isEmpty();
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of PublicKernelCircuitPublicInputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PublicKernelCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new PublicKernelCircuitPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(ValidationRequests),
      reader.readObject(PublicAccumulatedData),
      reader.readObject(PublicAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readObject(RevertCode),
    );
  }

  static empty() {
    return new PublicKernelCircuitPublicInputs(
      AggregationObject.makeFake(),
      ValidationRequests.empty(),
      PublicAccumulatedData.empty(),
      PublicAccumulatedData.empty(),
      CombinedConstantData.empty(),
      RevertCode.OK,
    );
  }

  [inspect.custom]() {
    return `PublicKernelCircuitPublicInputs {
      aggregationObject: ${this.aggregationObject},
      validationRequests: ${inspect(this.validationRequests)},
      endNonRevertibleData: ${inspect(this.endNonRevertibleData)},
      end: ${inspect(this.end)},
      constants: ${this.constants},
      revertCode: ${this.revertCode}
      }`;
  }
}

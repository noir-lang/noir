import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { AccumulatedNonRevertibleData, CombinedAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

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
     * Accumulated side effects and enqueued calls that are not revertible.
     */
    public endNonRevertibleData: AccumulatedNonRevertibleData,
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: CombinedAccumulatedData,
    /**
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
    /**
     * Indicates whether the input is for a private or public kernel.
     */
    public isPrivate: boolean,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.aggregationObject,
      this.endNonRevertibleData,
      this.end,
      this.constants,
      this.isPrivate,
    );
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
      reader.readObject(AccumulatedNonRevertibleData),
      reader.readObject(CombinedAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readBoolean(),
    );
  }

  static empty() {
    return new PublicKernelCircuitPublicInputs(
      AggregationObject.makeFake(),
      AccumulatedNonRevertibleData.empty(),
      CombinedAccumulatedData.empty(),
      CombinedConstantData.empty(),
      true,
    );
  }
}

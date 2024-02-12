import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { AccumulatedNonRevertibleData, FinalAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Output from to the private kernel circuit - tail call.
 */
export class PrivateKernelTailCircuitPublicInputs {
  constructor(
    /**
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
    /**
     * Accumulated side effects that are not revertible.
     */
    public endNonRevertibleData: AccumulatedNonRevertibleData,
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: FinalAccumulatedData,
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
   * @returns A new instance of PrivateKernelTailCircuitPublicInputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelTailCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelTailCircuitPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(AccumulatedNonRevertibleData),
      reader.readObject(FinalAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readBoolean(),
    );
  }

  static empty() {
    return new PrivateKernelTailCircuitPublicInputs(
      AggregationObject.makeFake(),
      AccumulatedNonRevertibleData.empty(),
      FinalAccumulatedData.empty(),
      CombinedConstantData.empty(),
      true,
    );
  }
}

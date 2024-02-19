import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import {
  CombinedAccumulatedData,
  PublicAccumulatedNonRevertibleData,
  PublicAccumulatedRevertibleData,
} from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Outputs from the public kernel circuits.
 * All Public kernels use this shape for outputs.
 */
export class PublicKernelCircuitPublicInputs {
  private combined: CombinedAccumulatedData | undefined = undefined;

  constructor(
    /**
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
    /**
     * Accumulated side effects and enqueued calls that are not revertible.
     */
    public endNonRevertibleData: PublicAccumulatedNonRevertibleData,
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: PublicAccumulatedRevertibleData,
    /**
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
    /**
     * Indicates whether the setup kernel is needed.
     */
    public needsSetup: boolean,
    /**
     * Indicates whether the app logic kernel is needed.
     */
    public needsAppLogic: boolean,
    /**
     * Indicates whether the teardown kernel is needed.
     */
    public needsTeardown: boolean,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.aggregationObject,
      this.endNonRevertibleData,
      this.end,
      this.constants,
      this.needsSetup,
      this.needsAppLogic,
      this.needsTeardown,
    );
  }

  get combinedData() {
    if (!this.combined) {
      this.combined = CombinedAccumulatedData.recombine(this.endNonRevertibleData, this.end);
    }
    return this.combined;
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
      reader.readObject(PublicAccumulatedNonRevertibleData),
      reader.readObject(PublicAccumulatedRevertibleData),
      reader.readObject(CombinedConstantData),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readBoolean(),
    );
  }

  static empty() {
    return new PublicKernelCircuitPublicInputs(
      AggregationObject.makeFake(),
      PublicAccumulatedNonRevertibleData.empty(),
      PublicAccumulatedRevertibleData.empty(),
      CombinedConstantData.empty(),
      true,
      true,
      true,
    );
  }
}

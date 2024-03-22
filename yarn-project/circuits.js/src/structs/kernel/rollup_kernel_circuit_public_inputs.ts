import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { RollupValidationRequests } from '../rollup_validation_requests.js';
import { CombinedAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Outputs from the public kernel circuits.
 * All Public kernels use this shape for outputs.
 */
export class RollupKernelCircuitPublicInputs {
  constructor(
    /**
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: CombinedAccumulatedData,
    /**
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
    /**
     * Validation requests accumulated from private and public execution to be completed by the rollup.
     */
    public rollupValidationRequests: RollupValidationRequests,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.aggregationObject, this.end, this.constants, this.rollupValidationRequests);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of RollupKernelCircuitPublicInputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): RollupKernelCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new RollupKernelCircuitPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(CombinedAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readObject(RollupValidationRequests),
    );
  }

  static empty() {
    return new RollupKernelCircuitPublicInputs(
      AggregationObject.makeFake(),
      CombinedAccumulatedData.empty(),
      CombinedConstantData.empty(),
      RollupValidationRequests.empty(),
    );
  }
}

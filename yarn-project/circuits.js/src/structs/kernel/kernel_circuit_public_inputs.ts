import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { RevertCode } from '../revert_code.js';
import { RollupValidationRequests } from '../rollup_validation_requests.js';
import { CombinedAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Outputs from the public kernel circuits.
 * All Public kernels use this shape for outputs.
 */
export class KernelCircuitPublicInputs {
  constructor(
    /**
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
    /**
     * Validation requests accumulated from private and public execution to be completed by the rollup.
     */
    public rollupValidationRequests: RollupValidationRequests,
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: CombinedAccumulatedData,
    /**
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
    /**
     * Flag indicating whether the transaction reverted.
     */
    public revertCode: RevertCode,
  ) {}

  getNonEmptyNullifiers() {
    return this.end.newNullifiers.filter(n => !n.isEmpty());
  }

  toBuffer() {
    return serializeToBuffer(
      this.aggregationObject,
      this.rollupValidationRequests,
      this.end,
      this.constants,
      this.revertCode,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of KernelCircuitPublicInputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): KernelCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new KernelCircuitPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(RollupValidationRequests),
      reader.readObject(CombinedAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readObject(RevertCode),
    );
  }

  static empty() {
    return new KernelCircuitPublicInputs(
      AggregationObject.makeFake(),
      RollupValidationRequests.empty(),
      CombinedAccumulatedData.empty(),
      CombinedConstantData.empty(),
      RevertCode.OK,
    );
  }
}

import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { RollupValidationRequests } from '../rollup_validation_requests.js';
import { PrivateAccumulatedNonRevertibleData, PrivateAccumulatedRevertibleData } from './combined_accumulated_data.js';
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
     * Validation requests for the rollup accumulated during private execution.
     */
    public rollupValidationRequests: RollupValidationRequests,
    /**
     * Accumulated side effects that are not revertible.
     */
    public endNonRevertibleData: PrivateAccumulatedNonRevertibleData,
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: PrivateAccumulatedRevertibleData,
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
      this.rollupValidationRequests,
      this.endNonRevertibleData,
      this.end,
      this.constants,
      this.needsSetup,
      this.needsAppLogic,
      this.needsTeardown,
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
      reader.readObject(RollupValidationRequests),
      reader.readObject(PrivateAccumulatedNonRevertibleData),
      reader.readObject(PrivateAccumulatedRevertibleData),
      reader.readObject(CombinedConstantData),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readBoolean(),
    );
  }

  static empty() {
    return new PrivateKernelTailCircuitPublicInputs(
      AggregationObject.makeFake(),
      RollupValidationRequests.empty(),
      PrivateAccumulatedNonRevertibleData.empty(),
      PrivateAccumulatedRevertibleData.empty(),
      CombinedConstantData.empty(),
      true,
      true,
      true,
    );
  }
}

import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { ValidationRequests } from '../validation_requests.js';
import { CombinedConstantData } from './combined_constant_data.js';
import { PrivateAccumulatedData } from './private_accumulated_data.js';

/**
 * Public inputs to the inner private kernel circuit
 */
export class PrivateKernelCircuitPublicInputs {
  constructor(
    /**
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
    /**
     * The side effect counter that non-revertible side effects are all beneath.
     */
    public minRevertibleSideEffectCounter: Fr,
    /**
     * Validation requests accumulated from public functions.
     */
    public validationRequests: ValidationRequests,
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: PrivateAccumulatedData,
    /**
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.aggregationObject,
      this.minRevertibleSideEffectCounter,
      this.validationRequests,
      this.end,
      this.constants,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of PrivateKernelInnerCircuitPublicInputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelCircuitPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(Fr),
      reader.readObject(ValidationRequests),
      reader.readObject(PrivateAccumulatedData),
      reader.readObject(CombinedConstantData),
    );
  }

  static empty() {
    return new PrivateKernelCircuitPublicInputs(
      AggregationObject.makeFake(),
      Fr.zero(),
      ValidationRequests.empty(),
      PrivateAccumulatedData.empty(),
      CombinedConstantData.empty(),
    );
  }
}

import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { CombinedAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Public inputs to the inner private kernel circuit
 */
export class PrivateKernelInnerCircuitPublicInputs {
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
      this.minRevertibleSideEffectCounter,
      this.end,
      this.constants,
      this.isPrivate,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of PrivateKernelInnerCircuitPublicInputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelInnerCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelInnerCircuitPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(Fr),
      reader.readObject(CombinedAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readBoolean(),
    );
  }

  static empty() {
    return new PrivateKernelInnerCircuitPublicInputs(
      AggregationObject.makeFake(),
      Fr.zero(),
      CombinedAccumulatedData.empty(),
      CombinedConstantData.empty(),
      true,
    );
  }
}

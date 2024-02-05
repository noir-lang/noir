import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { AccumulatedMetaData, FinalAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Public inputs of the final ordering private kernel circuit.
 */
export class KernelCircuitPublicInputsFinal {
  constructor(
    /**
     * Aggregated proof of all the previous kernel iterations.
     */
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations
    /**
     * The side effect counter that meta side effects are all beneath.
     */
    public metaHwm: Fr,
    /**
     * Final metadata accumulated for ordering private kernel circuit.
     */
    public endMeta: AccumulatedMetaData,
    /**
     * Final data accumulated for ordering private kernel circuit.
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
      this.metaHwm,
      this.endMeta,
      this.end,
      this.constants,
      this.isPrivate,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of KernelCircuitPublicInputsFinal.
   */
  static fromBuffer(buffer: Buffer | BufferReader): KernelCircuitPublicInputsFinal {
    const reader = BufferReader.asReader(buffer);
    return new KernelCircuitPublicInputsFinal(
      reader.readObject(AggregationObject),
      reader.readObject(Fr),
      reader.readObject(AccumulatedMetaData),
      reader.readObject(FinalAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readBoolean(),
    );
  }

  static empty() {
    return new KernelCircuitPublicInputsFinal(
      AggregationObject.makeFake(),
      Fr.zero(),
      AccumulatedMetaData.empty(),
      FinalAccumulatedData.empty(),
      CombinedConstantData.empty(),
      true,
    );
  }
}

/**
 * Public inputs of the final private kernel circuit.
 */
export class PrivateKernelPublicInputsFinal extends KernelCircuitPublicInputsFinal {
  constructor(
    aggregationObject: AggregationObject,
    metaHwm: Fr,
    endMeta: AccumulatedMetaData,
    end: FinalAccumulatedData,
    constants: CombinedConstantData,
  ) {
    super(aggregationObject, metaHwm, endMeta, end, constants, true);
  }
}

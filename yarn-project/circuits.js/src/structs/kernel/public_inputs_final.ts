import { BufferReader } from '@aztec/foundation/serialize';

import { serializeToBuffer } from '../../utils/serialize.js';
import { FinalAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Public inputs of the final ordering private kernel circuit.
 * @see circuits/cpp/src/aztec3/circuits/abis/kernel_circuit_public_inputs_final.hpp
 */
export class KernelCircuitPublicInputsFinal {
  constructor(
    /**
     * Final data accumulated for ordering privated kernel circuit.
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
    return serializeToBuffer(this.end, this.constants, this.isPrivate);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of KernelCircuitPublicInputsFinal.
   */
  static fromBuffer(buffer: Buffer | BufferReader): KernelCircuitPublicInputsFinal {
    const reader = BufferReader.asReader(buffer);
    return new KernelCircuitPublicInputsFinal(
      reader.readObject(FinalAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readBoolean(),
    );
  }

  static empty() {
    return new KernelCircuitPublicInputsFinal(FinalAccumulatedData.empty(), CombinedConstantData.empty(), true);
  }
}

/**
 * Public inputs of the final private kernel circuit.
 */
export class PrivateKernelPublicInputsFinal extends KernelCircuitPublicInputsFinal {
  constructor(end: FinalAccumulatedData, constants: CombinedConstantData) {
    super(end, constants, true);
  }
}

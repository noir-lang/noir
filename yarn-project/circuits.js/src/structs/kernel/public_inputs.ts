import { BufferReader } from '@aztec/foundation/serialize';

import { serializeToBuffer } from '../../utils/serialize.js';
import { CombinedAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Public inputs of the public and private kernel circuits.
 * @see circuits/cpp/src/aztec3/circuits/abis/kernel_circuit_public_inputs.hpp
 */
export class KernelCircuitPublicInputs {
  constructor(
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
    return serializeToBuffer(this.end, this.constants, this.isPrivate);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of KernelCircuitPublicInputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): KernelCircuitPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new KernelCircuitPublicInputs(
      reader.readObject(CombinedAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readBoolean(),
    );
  }

  static empty() {
    return new KernelCircuitPublicInputs(CombinedAccumulatedData.empty(), CombinedConstantData.empty(), true);
  }
}

/**
 * Public inputs of the public kernel circuit.
 */
export class PublicKernelPublicInputs extends KernelCircuitPublicInputs {
  constructor(end: CombinedAccumulatedData, constants: CombinedConstantData) {
    super(end, constants, false);
  }
}

/**
 * Public inputs of the private kernel circuit.
 */
export class PrivateKernelPublicInputs extends KernelCircuitPublicInputs {
  constructor(end: CombinedAccumulatedData, constants: CombinedConstantData) {
    super(end, constants, true);
  }
}

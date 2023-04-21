import { BufferReader } from '@aztec/foundation';
import { serializeToBuffer } from '../../utils/serialize.js';
import { CombinedAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';

/**
 * Public inputs of the public and private kernel circuits.
 * @see circuits/cpp/src/aztec3/circuits/abis/kernel_circuit_public_inputs.hpp
 */

export class KernelCircuitPublicInputs {
  constructor(
    public end: CombinedAccumulatedData,
    public constants: CombinedConstantData,
    public isPrivateKernel: boolean,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.end, this.constants, this.isPrivateKernel);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
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

export class PublicKernelPublicInputs extends KernelCircuitPublicInputs {
  constructor(end: CombinedAccumulatedData, constants: CombinedConstantData) {
    super(end, constants, false);
  }
}

export class PrivateKernelPublicInputs extends KernelCircuitPublicInputs {
  constructor(end: CombinedAccumulatedData, constants: CombinedConstantData) {
    super(end, constants, true);
  }
}

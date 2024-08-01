import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { PrivateValidationRequests } from '../private_validation_requests.js';
import { PublicCallRequest } from '../public_call_request.js';
import { CombinedConstantData } from './combined_constant_data.js';
import { PrivateAccumulatedData } from './private_accumulated_data.js';

/**
 * Public inputs to the inner private kernel circuit
 */
export class PrivateKernelCircuitPublicInputs {
  constructor(
    /**
     * The side effect counter that non-revertible side effects are all beneath.
     */
    public minRevertibleSideEffectCounter: Fr,
    /**
     * Validation requests accumulated from public functions.
     */
    public validationRequests: PrivateValidationRequests,
    /**
     * Data accumulated from both public and private circuits.
     */
    public end: PrivateAccumulatedData,
    /**
     * Data which is not modified by the circuits.
     */
    public constants: CombinedConstantData,
    /**
     * The call request for the public teardown function
     */
    public publicTeardownCallRequest: PublicCallRequest,
    /**
     * The address of the fee payer for the transaction
     */
    public feePayer: AztecAddress,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.minRevertibleSideEffectCounter,
      this.validationRequests,
      this.end,
      this.constants,
      this.publicTeardownCallRequest,
      this.feePayer,
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
      reader.readObject(Fr),
      reader.readObject(PrivateValidationRequests),
      reader.readObject(PrivateAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readObject(PublicCallRequest),
      reader.readObject(AztecAddress),
    );
  }

  static empty() {
    return new PrivateKernelCircuitPublicInputs(
      Fr.zero(),
      PrivateValidationRequests.empty(),
      PrivateAccumulatedData.empty(),
      CombinedConstantData.empty(),
      PublicCallRequest.empty(),
      AztecAddress.ZERO,
    );
  }
}

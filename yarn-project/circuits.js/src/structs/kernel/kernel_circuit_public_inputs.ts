import { AztecAddress } from '@aztec/foundation/aztec-address';
import type { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { type GasFees } from '../gas_fees.js';
import { PartialStateReference } from '../partial_state_reference.js';
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
    public startState: PartialStateReference,
    /**
     * Flag indicating whether the transaction reverted.
     */
    public revertCode: RevertCode,
    /**
     * The address of the fee payer for the transaction.
     */
    public feePayer: AztecAddress,
  ) {}

  getNonEmptyNullifiers() {
    return this.end.nullifiers.filter(n => !n.isZero());
  }

  /**
   * Computes the transaction fee for the transaction.
   * @param gasFees - Gas fees for the block. We cannot source this from the constants
   * since they may be unset if this comes from a private kernel directly.
   * @returns The amount in Fee Juice to pay for this tx.
   * @remarks It is safe to compute this method in typescript because we compute the
   * transaction_fee ourselves in the base rollup. This value must match the value
   * computed in the base rollup, otherwise the content commitment of the block will be invalid.
   */
  getTransactionFee(gasFees: GasFees): Fr {
    return this.end.gasUsed.computeFee(gasFees).add(this.constants.txContext.gasSettings.inclusionFee);
  }

  toBuffer() {
    return serializeToBuffer(
      this.rollupValidationRequests,
      this.end,
      this.constants,
      this.startState,
      this.revertCode,
      this.feePayer,
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
      reader.readObject(RollupValidationRequests),
      reader.readObject(CombinedAccumulatedData),
      reader.readObject(CombinedConstantData),
      reader.readObject(PartialStateReference),
      reader.readObject(RevertCode),
      reader.readObject(AztecAddress),
    );
  }

  static empty() {
    return new KernelCircuitPublicInputs(
      RollupValidationRequests.empty(),
      CombinedAccumulatedData.empty(),
      CombinedConstantData.empty(),
      PartialStateReference.empty(),
      RevertCode.OK,
      AztecAddress.ZERO,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static fromString(str: string) {
    return KernelCircuitPublicInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}

import { type FunctionCall } from '@aztec/circuit-types';
import { type GasSettings } from '@aztec/circuits.js';
import { computeSecretHash } from '@aztec/circuits.js/hash';
import { FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { type Wallet } from '../account/wallet.js';
import { type FeePaymentMethod } from './fee_payment_method.js';

/**
 * Holds information about how the fee for a transaction is to be paid.
 */
export class PrivateFeePaymentMethod implements FeePaymentMethod {
  constructor(
    /**
     * The asset used to pay the fee.
     */
    private asset: AztecAddress,
    /**
     * Address which will hold the fee payment.
     */
    private paymentContract: AztecAddress,

    /**
     * An auth witness provider to authorize fee payments
     */
    private wallet: Wallet,

    /**
     * A secret to shield the rebate amount from the FPC.
     * Use this to claim the shielded amount to private balance
     */
    private rebateSecret = Fr.random(),
  ) {}

  /**
   * The asset used to pay the fee.
   * @returns The asset used to pay the fee.
   */
  getAsset() {
    return this.asset;
  }

  getFeePayer(): Promise<AztecAddress> {
    return Promise.resolve(this.paymentContract);
  }

  /**
   * Creates a function call to pay the fee in the given asset.
   * @param gasSettings - The gas settings.
   * @returns The function call to pay the fee.
   */
  async getFunctionCalls(gasSettings: GasSettings): Promise<FunctionCall[]> {
    const nonce = Fr.random();
    const maxFee = gasSettings.getFeeLimit();
    await this.wallet.createAuthWit({
      caller: this.paymentContract,
      action: {
        name: 'unshield',
        args: [this.wallet.getCompleteAddress().address, this.paymentContract, maxFee, nonce],
        selector: FunctionSelector.fromSignature('unshield((Field),(Field),Field,Field)'),
        type: FunctionType.PRIVATE,
        isStatic: false,
        to: this.asset,
        returnTypes: [],
      },
    });

    const secretHashForRebate = computeSecretHash(this.rebateSecret);

    return [
      {
        name: 'fee_entrypoint_private',
        to: this.paymentContract,
        selector: FunctionSelector.fromSignature('fee_entrypoint_private(Field,(Field),Field,Field)'),
        type: FunctionType.PRIVATE,
        isStatic: false,
        args: [maxFee, this.asset, secretHashForRebate, nonce],
        returnTypes: [],
      },
    ];
  }
}

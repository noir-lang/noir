import { type FunctionCall } from '@aztec/circuit-types';
import { FunctionData, type GasSettings } from '@aztec/circuits.js';
import { computeSecretHash } from '@aztec/circuits.js/hash';
import { FunctionSelector } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { type Wallet } from '../account/wallet.js';
import { computeAuthWitMessageHash } from '../utils/authwit.js';
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

  /**
   * The address which will facilitate the fee payment.
   * @returns The contract address responsible for holding the fee payment.
   */
  getPaymentContract() {
    return this.paymentContract;
  }

  /**
   * Creates a function call to pay the fee in the given asset.
   * @param gasSettings - The gas settings.
   * @returns The function call to pay the fee.
   */
  async getFunctionCalls(gasSettings: GasSettings): Promise<FunctionCall[]> {
    const nonce = Fr.random();
    const maxFee = gasSettings.getFeeLimit();
    const messageHash = computeAuthWitMessageHash(
      this.paymentContract,
      this.wallet.getChainId(),
      this.wallet.getVersion(),
      {
        args: [this.wallet.getCompleteAddress().address, this.paymentContract, maxFee, nonce],
        functionData: new FunctionData(FunctionSelector.fromSignature('unshield((Field),(Field),Field,Field)'), true),
        to: this.asset,
      },
    );
    await this.wallet.createAuthWit(messageHash);

    const secretHashForRebate = computeSecretHash(this.rebateSecret);

    return [
      {
        to: this.getPaymentContract(),
        functionData: new FunctionData(
          FunctionSelector.fromSignature('fee_entrypoint_private(Field,(Field),Field,Field)'),
          true,
        ),
        args: [maxFee, this.asset, secretHashForRebate, nonce],
      },
    ];
  }
}

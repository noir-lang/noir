import { FunctionCall } from '@aztec/circuit-types';
import { FunctionData } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { computeAuthWitMessageHash } from '../utils/authwit.js';
import { AccountWalletWithPrivateKey } from '../wallet/account_wallet_with_private_key.js';
import { FeePaymentMethod } from './fee_payment_method.js';

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
    private wallet: AccountWalletWithPrivateKey,
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
   * @param maxFee - The maximum fee to be paid in the given asset.
   * @returns The function call to pay the fee.
   */
  async getFunctionCalls(maxFee: Fr): Promise<FunctionCall[]> {
    const nonce = Fr.random();
    const messageHash = computeAuthWitMessageHash(this.paymentContract, {
      args: [this.wallet.getAddress(), this.paymentContract, maxFee, nonce],
      functionData: new FunctionData(
        FunctionSelector.fromSignature('unshield((Field),(Field),Field,Field)'),
        false,
        true,
        false,
      ),
      to: this.asset,
    });
    await this.wallet.createAuthWitness(messageHash);

    return [
      {
        to: this.getPaymentContract(),
        functionData: new FunctionData(
          FunctionSelector.fromSignature('fee_entrypoint_private(Field,(Field),Field)'),
          false,
          true,
          false,
        ),
        args: [maxFee, this.asset, nonce],
      },
    ];
  }
}

import { type FunctionCall } from '@aztec/circuit-types';
import { FunctionData } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { computeAuthWitMessageHash } from '../utils/authwit.js';
import { type AccountWallet } from '../wallet/account_wallet.js';
import { type FeePaymentMethod } from './fee_payment_method.js';

/**
 * Holds information about how the fee for a transaction is to be paid.
 */
export class PublicFeePaymentMethod implements FeePaymentMethod {
  constructor(
    /**
     * The asset used to pay the fee.
     */
    protected asset: AztecAddress,
    /**
     * Address which will hold the fee payment.
     */
    protected paymentContract: AztecAddress,

    /**
     * An auth witness provider to authorize fee payments
     */
    protected wallet: AccountWallet,
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
  getFunctionCalls(maxFee: Fr): Promise<FunctionCall[]> {
    const nonce = Fr.random();

    const messageHash = computeAuthWitMessageHash(
      this.paymentContract,
      this.wallet.getChainId(),
      this.wallet.getVersion(),
      {
        args: [this.wallet.getAddress(), this.paymentContract, maxFee, nonce],
        functionData: new FunctionData(
          FunctionSelector.fromSignature('transfer_public((Field),(Field),Field,Field)'),
          false,
        ),
        to: this.asset,
      },
    );

    return Promise.resolve([
      this.wallet.setPublicAuthWit(messageHash, true).request(),
      {
        to: this.getPaymentContract(),
        functionData: new FunctionData(
          FunctionSelector.fromSignature('fee_entrypoint_public(Field,(Field),Field)'),
          true,
        ),
        args: [maxFee, this.asset, nonce],
      },
    ]);
  }
}

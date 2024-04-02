import { type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress, FunctionData } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { type Fr } from '@aztec/foundation/fields';
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { type Wallet } from '../account/wallet.js';
import { type FeePaymentMethod } from './fee_payment_method.js';

/**
 * Pay fee directly in the native gas token.
 */
export class NativeFeePaymentMethod implements FeePaymentMethod {
  #gasTokenAddress: AztecAddress;

  private constructor(canonicalGasTokenAddress: AztecAddress) {
    this.#gasTokenAddress = canonicalGasTokenAddress;
  }

  static async create(wallet: Wallet): Promise<NativeFeePaymentMethod> {
    const nodeInfo = await wallet.getNodeInfo();
    return new NativeFeePaymentMethod(getCanonicalGasTokenAddress(nodeInfo.l1ContractAddresses.gasPortalAddress));
  }

  /**
   * Gets the native gas asset used to pay the fee.
   * @returns The asset used to pay the fee.
   */
  getAsset() {
    return this.#gasTokenAddress;
  }

  /**
   * The contract responsible for fee payment. This will be the same as the asset.
   * @returns The contract address responsible for holding the fee payment.
   */
  getPaymentContract() {
    return this.#gasTokenAddress;
  }

  /**
   * Fee payments in the native gas token are always public.
   * @returns false
   */
  isPrivateFeePayment(): boolean {
    return false;
  }

  /**
   * Creates a function call to pay the fee in gas token..
   * @param feeLimit - The maximum fee to be paid in gas token.
   * @returns A function call
   */
  getFunctionCalls(feeLimit: Fr): Promise<FunctionCall[]> {
    return Promise.resolve([
      {
        to: this.#gasTokenAddress,
        functionData: new FunctionData(FunctionSelector.fromSignature('pay_fee(Field)'), false),
        args: [feeLimit],
      },
    ]);
  }
}

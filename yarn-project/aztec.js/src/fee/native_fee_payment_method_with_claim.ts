import { type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress, Fr, FunctionSelector } from '@aztec/circuits.js';
import { FunctionType } from '@aztec/foundation/abi';
import { GasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { NativeFeePaymentMethod } from './native_fee_payment_method.js';

/**
 * Pay fee directly with native gas token claimed on the same tx.
 */
export class NativeFeePaymentMethodWithClaim extends NativeFeePaymentMethod {
  constructor(sender: AztecAddress, private claimAmount: bigint | Fr, private claimSecret: Fr) {
    super(sender);
  }

  /**
   * Creates a function call to pay the fee in gas token.
   * @returns A function call
   */
  override getFunctionCalls(): Promise<FunctionCall[]> {
    return Promise.resolve([
      {
        to: GasTokenAddress,
        name: 'claim',
        selector: FunctionSelector.fromSignature('claim((Field),Field,Field)'),
        isStatic: false,
        args: [this.sender, new Fr(this.claimAmount), this.claimSecret],
        returnTypes: [],
        type: FunctionType.PRIVATE,
      },
    ]);
  }
}

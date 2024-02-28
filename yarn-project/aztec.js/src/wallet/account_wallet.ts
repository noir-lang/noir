import { AuthWitness, FunctionCall, PXE, TxExecutionRequest } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { ABIParameterVisibility, FunctionAbi, FunctionType } from '@aztec/foundation/abi';

import { AccountInterface, FeeOptions } from '../account/interface.js';
import { ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { BaseWallet } from './base_wallet.js';

/**
 * A wallet implementation that forwards authentication requests to a provided account.
 */
export class AccountWallet extends BaseWallet {
  constructor(pxe: PXE, protected account: AccountInterface) {
    super(pxe);
  }

  createTxExecutionRequest(execs: FunctionCall[], fee?: FeeOptions): Promise<TxExecutionRequest> {
    return this.account.createTxExecutionRequest(execs, fee);
  }

  async createAuthWitness(message: Fr | Buffer): Promise<AuthWitness> {
    message = Buffer.isBuffer(message) ? Fr.fromBuffer(message) : message;
    const witness = await this.account.createAuthWitness(message);
    await this.pxe.addAuthWitness(witness);
    return witness;
  }

  /**
   * Returns a function interaction to set a message hash as authorized in this account.
   * Public calls can then consume this authorization.
   * @param message - Message hash to authorize.
   * @param authorized - True to authorize, false to revoke authorization.
   * @returns - A function interaction.
   */
  public setPublicAuth(message: Fr | Buffer, authorized: boolean): ContractFunctionInteraction {
    if (authorized) {
      return new ContractFunctionInteraction(this, this.getAddress(), this.getApprovePublicAuthwitAbi(), [message]);
    } else {
      return this.cancelAuthWit(message);
    }
  }

  /**
   * Returns a function interaction to cancel a message hash as authorized in this account.
   * @param message - Message hash to authorize.
   * @returns - A function interaction.
   */
  public cancelAuthWit(message: Fr | Buffer): ContractFunctionInteraction {
    const args = [message];
    return new ContractFunctionInteraction(this, this.getAddress(), this.getCancelAuthwitAbi(), args);
  }

  /** Returns the complete address of the account that implements this wallet. */
  public getCompleteAddress() {
    return this.account.getCompleteAddress();
  }

  /** Returns the address of the account that implements this wallet. */
  public getAddress() {
    return this.getCompleteAddress().address;
  }

  private getApprovePublicAuthwitAbi(): FunctionAbi {
    return {
      name: 'approve_public_authwit',
      functionType: FunctionType.OPEN,
      isInternal: true,
      parameters: [
        {
          name: 'message_hash',
          type: { kind: 'field' },
          visibility: 'private' as ABIParameterVisibility,
        },
      ],
      returnTypes: [],
    };
  }

  private getCancelAuthwitAbi(): FunctionAbi {
    return {
      name: 'cancel_authwit',
      functionType: FunctionType.SECRET,
      isInternal: true,
      parameters: [
        {
          name: 'message_hash',
          type: { kind: 'field' },
          visibility: 'private' as ABIParameterVisibility,
        },
      ],
      returnTypes: [],
    };
  }
}

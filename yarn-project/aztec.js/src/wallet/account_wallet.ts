import { AuthWitness, FunctionCall, PXE, TxExecutionRequest } from '@aztec/circuit-types';
import { AztecAddress, Fr } from '@aztec/circuits.js';
import { ABIParameterVisibility, FunctionAbi, FunctionType } from '@aztec/foundation/abi';

import { AccountInterface, FeeOptions } from '../account/interface.js';
import { ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { computeAuthWitMessageHash } from '../utils/authwit.js';
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

  /**
   * Computes an authentication witness from either a message or a caller and an action.
   * If a message is provided, it will create a witness for the message directly.
   * Otherwise, it will compute the message using the caller and the action.
   * @param messageHashOrIntent - The message or the caller and action to approve
   * @returns The authentication witness
   */
  async createAuthWit(
    messageHashOrIntent:
      | Fr
      | Buffer
      | {
          /** The caller to approve  */
          caller: AztecAddress;
          /** The action to approve */
          action: ContractFunctionInteraction | FunctionCall;
        },
  ): Promise<AuthWitness> {
    const messageHash = this.getMessageHash(messageHashOrIntent);
    const witness = await this.account.createAuthWit(messageHash);
    await this.pxe.addAuthWitness(witness);
    return witness;
  }

  /**
   * Returns the message hash for the given message or authwit input.
   * @param messageHashOrIntent - The message hash or the caller and action to authorize
   * @returns The message hash
   */
  private getMessageHash(
    messageHashOrIntent:
      | Fr
      | Buffer
      | {
          /** The caller to approve  */
          caller: AztecAddress;
          /** The action to approve */
          action: ContractFunctionInteraction | FunctionCall;
        },
  ): Fr {
    if (Buffer.isBuffer(messageHashOrIntent)) {
      return Fr.fromBuffer(messageHashOrIntent);
    } else if (messageHashOrIntent instanceof Fr) {
      return messageHashOrIntent;
    } else if (messageHashOrIntent.action instanceof ContractFunctionInteraction) {
      return computeAuthWitMessageHash(messageHashOrIntent.caller, messageHashOrIntent.action.request());
    }
    return computeAuthWitMessageHash(messageHashOrIntent.caller, messageHashOrIntent.action);
  }

  /**
   * Returns a function interaction to set a message hash as authorized or revoked in this account.
   * Public calls can then consume this authorization.
   * @param messageHashOrIntent - The message or the caller and action to authorize/revoke
   * @param authorized - True to authorize, false to revoke authorization.
   * @returns - A function interaction.
   */
  public setPublicAuthWit(
    messageHashOrIntent:
      | Fr
      | Buffer
      | {
          /** The caller to approve  */
          caller: AztecAddress;
          /** The action to approve */
          action: ContractFunctionInteraction | FunctionCall;
        },
    authorized: boolean,
  ): ContractFunctionInteraction {
    const message = this.getMessageHash(messageHashOrIntent);
    if (authorized) {
      return new ContractFunctionInteraction(this, this.getAddress(), this.getApprovePublicAuthwitAbi(), [message]);
    } else {
      return this.cancelAuthWit(message);
    }
  }

  /**
   * Returns a function interaction to cancel a message hash as authorized in this account.
   * @param messageHashOrIntent - The message or the caller and action to authorize/revoke
   * @returns - A function interaction.
   */
  public cancelAuthWit(
    messageHashOrIntent:
      | Fr
      | Buffer
      | {
          /** The caller to approve  */
          caller: AztecAddress;
          /** The action to approve */
          action: ContractFunctionInteraction | FunctionCall;
        },
  ): ContractFunctionInteraction {
    const message = this.getMessageHash(messageHashOrIntent);
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
      isInitializer: false,
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
      isInitializer: false,
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

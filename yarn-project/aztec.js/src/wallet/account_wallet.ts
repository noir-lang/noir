import { type AuthWitness, type FunctionCall, type PXE, type TxExecutionRequest } from '@aztec/circuit-types';
import { type AztecAddress, Fr } from '@aztec/circuits.js';
import { type ABIParameterVisibility, type FunctionAbi, FunctionType } from '@aztec/foundation/abi';

import { type AccountInterface } from '../account/interface.js';
import { ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { type FeeOptions } from '../entrypoint/entrypoint.js';
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

  getChainId(): Fr {
    return this.account.getChainId();
  }

  getVersion(): Fr {
    return this.account.getVersion();
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
          /** The chain id to approve */
          chainId?: Fr;
          /** The version to approve  */
          version?: Fr;
        },
  ): Promise<AuthWitness> {
    const messageHash = this.getMessageHash(messageHashOrIntent);
    const witness = await this.account.createAuthWit(messageHash);
    await this.pxe.addAuthWitness(witness);
    return witness;
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
          /** The chain id to approve */
          chainId?: Fr;
          /** The version to approve  */
          version?: Fr;
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
          /** The chain id to approve */
          chainId?: Fr;
          /** The version to approve  */
          version?: Fr;
        },
  ): Fr {
    if (Buffer.isBuffer(messageHashOrIntent)) {
      return Fr.fromBuffer(messageHashOrIntent);
    } else if (messageHashOrIntent instanceof Fr) {
      return messageHashOrIntent;
    } else {
      return computeAuthWitMessageHash(
        messageHashOrIntent.caller,
        messageHashOrIntent.chainId || this.getChainId(),
        messageHashOrIntent.version || this.getVersion(),
        messageHashOrIntent.action instanceof ContractFunctionInteraction
          ? messageHashOrIntent.action.request()
          : messageHashOrIntent.action,
      );
    }
  }

  /**
   * Lookup the validity of an authwit in private and public contexts.
   * If the authwit have been consumed already (nullifier spent), will return false in both contexts.
   * @param target - The target contract address
   * @param messageHashOrIntent - The message hash or the caller and action to authorize/revoke
   * @returns - A struct containing the validity of the authwit in private and public contexts.
   */
  async lookupValidity(
    target: AztecAddress,
    messageHashOrIntent:
      | Fr
      | Buffer
      | {
          /** The caller to approve  */
          caller: AztecAddress;
          /** The action to approve */
          action: ContractFunctionInteraction | FunctionCall;
          /** The chain id to approve */
          chainId?: Fr;
          /** The version to approve  */
          version?: Fr;
        },
  ): Promise<{
    /** boolean flag indicating if the authwit is valid in private context */
    isValidInPrivate: boolean;
    /** boolean flag indicating if the authwit is valid in public context */
    isValidInPublic: boolean;
  }> {
    const messageHash = this.getMessageHash(messageHashOrIntent);
    const witness = await this.getAuthWitness(messageHash);
    const blockNumber = await this.getBlockNumber();
    const interaction = new ContractFunctionInteraction(this, target, this.getLookupValidityAbi(), [
      target,
      blockNumber,
      witness != undefined,
      messageHash,
    ]);

    const [isValidInPrivate, isValidInPublic] = (await interaction.simulate()) as [boolean, boolean];
    return { isValidInPrivate, isValidInPublic };
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
          /** The chain id to approve */
          chainId?: Fr;
          /** The version to approve  */
          version?: Fr;
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

  private getLookupValidityAbi(): FunctionAbi {
    return {
      name: 'lookup_validity',
      isInitializer: false,
      functionType: FunctionType.UNCONSTRAINED,
      isInternal: false,
      parameters: [
        {
          name: 'myself',
          type: {
            kind: 'struct',
            path: 'authwit::aztec::protocol_types::address::aztec_address::AztecAddress',
            fields: [{ name: 'inner', type: { kind: 'field' } }],
          },
          visibility: 'private' as ABIParameterVisibility,
        },
        {
          name: 'block_number',
          type: { kind: 'integer', sign: 'unsigned', width: 32 },
          visibility: 'private' as ABIParameterVisibility,
        },
        {
          name: 'check_private',
          type: { kind: 'boolean' },
          visibility: 'private' as ABIParameterVisibility,
        },
        { name: 'message_hash', type: { kind: 'field' }, visibility: 'private' as ABIParameterVisibility },
      ],
      returnTypes: [{ kind: 'array', length: 2, type: { kind: 'boolean' } }],
    };
  }
}

import { type AuthWitness, type PXE, type TxExecutionRequest } from '@aztec/circuit-types';
import { AztecAddress, CANONICAL_KEY_REGISTRY_ADDRESS, Fq, Fr, derivePublicKeyFromSecretKey } from '@aztec/circuits.js';
import { type ABIParameterVisibility, type FunctionAbi, FunctionType } from '@aztec/foundation/abi';
import { AuthRegistryAddress } from '@aztec/protocol-contracts/auth-registry';

import { type AccountInterface } from '../account/interface.js';
import { ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { type ExecutionRequestInit } from '../entrypoint/entrypoint.js';
import {
  type IntentAction,
  type IntentInnerHash,
  computeAuthWitMessageHash,
  computeInnerAuthWitHashFromAction,
} from '../utils/authwit.js';
import { BaseWallet } from './base_wallet.js';

/**
 * A wallet implementation that forwards authentication requests to a provided account.
 */
export class AccountWallet extends BaseWallet {
  constructor(pxe: PXE, protected account: AccountInterface) {
    super(pxe);
  }

  createTxExecutionRequest(exec: ExecutionRequestInit): Promise<TxExecutionRequest> {
    return this.account.createTxExecutionRequest(exec);
  }

  getChainId(): Fr {
    return this.account.getChainId();
  }

  getVersion(): Fr {
    return this.account.getVersion();
  }

  /**
   * Computes an authentication witness from either a message hash or an intent.
   *
   * If a message hash is provided, it will create a witness for the hash directly.
   * Otherwise, it will compute the message hash using the intent, along with the
   * chain id and the version values provided by the wallet.
   *
   * @param messageHashOrIntent - The message hash of the intent to approve
   * @returns The authentication witness
   */
  async createAuthWit(messageHashOrIntent: Fr | Buffer | IntentAction | IntentInnerHash): Promise<AuthWitness> {
    let messageHash: Fr;
    if (Buffer.isBuffer(messageHashOrIntent)) {
      messageHash = Fr.fromBuffer(messageHashOrIntent);
    } else if (messageHashOrIntent instanceof Fr) {
      messageHash = messageHashOrIntent;
    } else {
      messageHash = this.getMessageHash(messageHashOrIntent);
    }

    const witness = await this.account.createAuthWit(messageHash);
    await this.pxe.addAuthWitness(witness);
    return witness;
  }

  /**
   * Returns a function interaction to set a message hash as authorized or revoked in this account.
   *
   * Public calls can then consume this authorization.
   *
   * @param messageHashOrIntent - The message hash or intent to authorize/revoke
   * @param authorized - True to authorize, false to revoke authorization.
   * @returns - A function interaction.
   */
  public setPublicAuthWit(
    messageHashOrIntent: Fr | Buffer | IntentInnerHash | IntentAction,
    authorized: boolean,
  ): ContractFunctionInteraction {
    let messageHash: Fr;
    if (Buffer.isBuffer(messageHashOrIntent)) {
      messageHash = Fr.fromBuffer(messageHashOrIntent);
    } else if (messageHashOrIntent instanceof Fr) {
      messageHash = messageHashOrIntent;
    } else {
      messageHash = this.getMessageHash(messageHashOrIntent);
    }

    return new ContractFunctionInteraction(this, AuthRegistryAddress, this.getSetAuthorizedAbi(), [
      messageHash,
      authorized,
    ]);
  }

  private getInnerHashAndConsumer(intent: IntentInnerHash | IntentAction): {
    /** The inner hash */
    innerHash: Fr;
    /** The consumer of the authwit */
    consumer: AztecAddress;
  } {
    if ('caller' in intent && 'action' in intent) {
      const action = intent.action instanceof ContractFunctionInteraction ? intent.action.request() : intent.action;
      return {
        innerHash: computeInnerAuthWitHashFromAction(intent.caller, action),
        consumer: action.to,
      };
    } else if (Buffer.isBuffer(intent.innerHash)) {
      return { innerHash: Fr.fromBuffer(intent.innerHash), consumer: intent.consumer };
    }
    return { innerHash: intent.innerHash, consumer: intent.consumer };
  }

  /**
   * Returns the message hash for the given intent
   *
   * @param intent - A tuple of (consumer and inner hash) or (caller and action)
   * @returns The message hash
   */
  private getMessageHash(intent: IntentInnerHash | IntentAction): Fr {
    const chainId = this.getChainId();
    const version = this.getVersion();
    return computeAuthWitMessageHash(intent, { chainId, version });
  }

  /**
   * Lookup the validity of an authwit in private and public contexts.
   *
   * Uses the chain id and version of the wallet.
   *
   * @param onBehalfOf - The address of the "approver"
   * @param intent - The consumer and inner hash or the caller and action to lookup
   *
   * @returns - A struct containing the validity of the authwit in private and public contexts.
   */
  async lookupValidity(
    onBehalfOf: AztecAddress,
    intent: IntentInnerHash | IntentAction,
  ): Promise<{
    /** boolean flag indicating if the authwit is valid in private context */
    isValidInPrivate: boolean;
    /** boolean flag indicating if the authwit is valid in public context */
    isValidInPublic: boolean;
  }> {
    const { innerHash, consumer } = this.getInnerHashAndConsumer(intent);

    const messageHash = this.getMessageHash(intent);
    const results = { isValidInPrivate: false, isValidInPublic: false };

    // Check private
    const witness = await this.getAuthWitness(messageHash);
    if (witness !== undefined) {
      results.isValidInPrivate = (await new ContractFunctionInteraction(this, onBehalfOf, this.getLookupValidityAbi(), [
        consumer,
        innerHash,
      ]).simulate()) as boolean;
    }

    // check public
    results.isValidInPublic = (await new ContractFunctionInteraction(
      this,
      AuthRegistryAddress,
      this.getIsConsumableAbi(),
      [onBehalfOf, messageHash],
    ).simulate()) as boolean;

    return results;
  }

  /**
   * Rotates the account master nullifier key pair.
   * @param newNskM - The new master nullifier secret key we want to use.
   * @remarks - This function also calls the canonical key registry with the account's new derived master nullifier public key.
   * We are doing it this way to avoid user error, in the case that a user rotates their keys in the key registry,
   * but fails to do so in the key store. This leads to unspendable notes.
   *
   * This does not hinder our ability to spend notes tied to a previous master nullifier public key, provided we have the master nullifier secret key for it.
   */
  public async rotateNullifierKeys(newNskM: Fq = Fq.random()): Promise<void> {
    // We rotate our secret key in the keystore first, because if the subsequent interaction fails, there are no bad side-effects.
    // If vice versa (the key registry is called first), but the call to the PXE fails, we will end up in a situation with unspendable notes, as we have not committed our
    // nullifier secret key to our wallet.
    await this.pxe.rotateNskM(this.getAddress(), newNskM);
    const interaction = new ContractFunctionInteraction(
      this,
      AztecAddress.fromBigInt(CANONICAL_KEY_REGISTRY_ADDRESS),
      this.getRotateNpkMAbi(),
      [this.getAddress(), derivePublicKeyFromSecretKey(newNskM).toNoirStruct(), Fr.ZERO],
    );

    await interaction.send().wait();
  }

  /** Returns the complete address of the account that implements this wallet. */
  public getCompleteAddress() {
    return this.account.getCompleteAddress();
  }

  /** Returns the address of the account that implements this wallet. */
  public override getAddress() {
    return this.getCompleteAddress().address;
  }

  private getSetAuthorizedAbi(): FunctionAbi {
    return {
      name: 'set_authorized',
      isInitializer: false,
      functionType: FunctionType.PUBLIC,
      isInternal: true,
      isStatic: false,
      parameters: [
        {
          name: 'message_hash',
          type: { kind: 'field' },
          visibility: 'private' as ABIParameterVisibility,
        },
        {
          name: 'authorize',
          type: { kind: 'boolean' },
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
      isStatic: false,
      parameters: [{ name: 'message_hash', type: { kind: 'field' }, visibility: 'private' as ABIParameterVisibility }],
      returnTypes: [{ kind: 'boolean' }],
    };
  }

  private getIsConsumableAbi(): FunctionAbi {
    return {
      name: 'unconstrained_is_consumable',
      isInitializer: false,
      functionType: FunctionType.UNCONSTRAINED,
      isInternal: false,
      isStatic: false,
      parameters: [
        {
          name: 'address',
          type: {
            fields: [{ name: 'inner', type: { kind: 'field' } }],
            kind: 'struct',
            path: 'authwit::aztec::protocol_types::address::aztec_address::AztecAddress',
          },
          visibility: 'private' as ABIParameterVisibility,
        },
        { name: 'message_hash', type: { kind: 'field' }, visibility: 'private' as ABIParameterVisibility },
      ],
      returnTypes: [{ kind: 'boolean' }],
    };
  }

  private getRotateNpkMAbi(): FunctionAbi {
    return {
      name: 'rotate_npk_m',
      isInitializer: false,
      functionType: FunctionType.PUBLIC,
      isInternal: false,
      isStatic: false,
      parameters: [
        {
          name: 'address',
          type: {
            fields: [{ name: 'inner', type: { kind: 'field' } }],
            kind: 'struct',
            path: 'authwit::aztec::protocol_types::address::aztec_address::AztecAddress',
          },
          visibility: 'private' as ABIParameterVisibility,
        },
        {
          name: 'new_npk_m',
          type: {
            fields: [
              { name: 'x', type: { kind: 'field' } },
              { name: 'y', type: { kind: 'field' } },
              { name: 'is_infinite', type: { kind: 'boolean' } },
            ],
            kind: 'struct',
            path: 'std::embedded_curve_ops::EmbeddedCurvePoint',
          },
          visibility: 'private' as ABIParameterVisibility,
        },
        { name: 'nonce', type: { kind: 'field' }, visibility: 'private' as ABIParameterVisibility },
      ],
      returnTypes: [],
    };
  }
}

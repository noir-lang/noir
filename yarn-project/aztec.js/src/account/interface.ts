import { type AuthWitness, type CompleteAddress, type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { type Fq, type Fr } from '@aztec/foundation/fields';

import { type ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { type EntrypointInterface } from '../entrypoint/entrypoint.js';

// docs:start:account-interface
/** Creates authorization witnesses. */
export interface AuthWitnessProvider {
  /**
   * Computes an authentication witness from either a message hash or an intent (caller and an action).
   * If a message hash is provided, it will create a witness for that directly.
   * Otherwise, it will compute the message hash using the caller and the action of the intent.
   * @param messageHashOrIntent - The message hash or the intent (caller and action) to approve
   * @param chainId - The chain id for the message, will default to the current chain id
   * @param version - The version for the message, will default to the current protocol version
   * @returns The authentication witness
   */
  createAuthWit(
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
  ): Promise<AuthWitness>;
}

/**
 * Handler for interfacing with an account. Knows how to create transaction execution
 * requests and authorize actions for its corresponding account.
 */
export interface AccountInterface extends AuthWitnessProvider, EntrypointInterface {
  /** Returns the complete address for this account. */
  getCompleteAddress(): CompleteAddress;

  /** Returns the address for this account. */
  getAddress(): AztecAddress;

  /** Returns the chain id for this account */
  getChainId(): Fr;

  /** Returns the rollup version for this account */
  getVersion(): Fr;
}

/**
 * Handler for interfacing with an account's ability to rotate its keys.
 */
export interface AccountKeyRotationInterface {
  /**
   * Rotates the account master nullifier key pair.
   * @param newNskM - The new master nullifier secret key we want to use.
   * @remarks - This function also calls the canonical key registry with the account's new derived master nullifier public key.
   * We are doing it this way to avoid user error, in the case that a user rotates their keys in the key registry,
   * but fails to do so in the key store. This leads to unspendable notes.
   *
   * This does not hinder our ability to spend notes tied to a previous master nullifier public key, provided we have the master nullifier secret key for it.
   */
  rotateNullifierKeys(newNskM: Fq): Promise<void>;
}
// docs:end:account-interface

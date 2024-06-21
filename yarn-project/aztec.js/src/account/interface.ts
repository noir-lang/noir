import { type AuthWitness, type CompleteAddress } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { type Fq, type Fr } from '@aztec/foundation/fields';

import { type EntrypointInterface } from '../entrypoint/entrypoint.js';

// docs:start:account-interface
/** Creates authorization witnesses. */
export interface AuthWitnessProvider {
  /**
   * Computes an authentication witness from either a message hash
   * @param messageHash - The message hash to approve
   * @returns The authentication witness
   */
  createAuthWit(messageHash: Fr | Buffer): Promise<AuthWitness>;
}

/**
 * Handler for interfacing with an account. Knows how to create transaction execution
 * requests and authorize actions for its corresponding account.
 */
export interface AccountInterface extends EntrypointInterface, AuthWitnessProvider {
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

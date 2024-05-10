import { type AuthWitness, type CompleteAddress, type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

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
// docs:end:account-interface

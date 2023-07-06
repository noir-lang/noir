import { AztecAddress, TxContext } from '@aztec/circuits.js';
import { ExecutionRequest, TxExecutionRequest } from '@aztec/types';
import { AccountImplementation } from './index.js';

/**
 * A concrete account implementation that manages multiple accounts.
 */
export class AccountCollection implements AccountImplementation {
  private accounts: Map<AztecAddress, AccountImplementation> = new Map();

  /**
   * Registers an account implementation against an aztec address
   * @param addr - The aztec address agianst which to register the implementation.
   * @param impl - The account implementation to be registered.
   */
  public registerAccount(addr: AztecAddress, impl: AccountImplementation) {
    this.accounts.set(addr, impl);
  }

  getAddress(): AztecAddress {
    if (!this.accounts) throw new Error(`No accounts registered`);
    return this.accounts.keys().next().value as AztecAddress;
  }

  /**
   * Uses a registered account implementation to generate an authenticated request
   * @param executions - The execution intent to be authenticated.
   * @param txContext - The tx context under with the execution is to be made.
   * @returns - The authenticated transaction execution request.
   */
  public createAuthenticatedTxRequest(
    executions: ExecutionRequest[],
    txContext: TxContext,
  ): Promise<TxExecutionRequest> {
    // TODO: Check all executions have the same origin
    const sender = executions[0].from;
    const impl = this.accounts.get(sender);
    if (!impl) throw new Error(`No account implementation registered for ${sender}`);
    return impl.createAuthenticatedTxRequest(executions, txContext);
  }
}

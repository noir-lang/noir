import { AztecAddress, TxContext } from '@aztec/circuits.js';
import { ExecutionRequest, TxExecutionRequest } from '@aztec/types';

import { AccountImplementation } from './index.js';

/**
 * A concrete account implementation that manages multiple accounts.
 */
export class AccountCollection implements AccountImplementation {
  private accounts: Map<string, AccountImplementation> = new Map();

  /**
   * Registers an account implementation against an aztec address
   * @param addr - The aztec address agianst which to register the implementation.
   * @param impl - The account implementation to be registered.
   */
  public registerAccount(addr: AztecAddress, impl: AccountImplementation) {
    this.accounts.set(addr.toString(), impl);
  }

  getAddress(): AztecAddress {
    if (!this.accounts) throw new Error(`No accounts registered`);
    return AztecAddress.fromString(this.accounts.keys().next().value as string);
  }

  public createAuthenticatedTxRequest(
    executions: ExecutionRequest[],
    txContext: TxContext,
  ): Promise<TxExecutionRequest> {
    // TODO: Check all executions have the same origin
    const sender = executions[0].from;
    const impl = this.accounts.get(sender.toString());
    if (!impl) throw new Error(`No account implementation registered for ${sender}`);
    return impl.createAuthenticatedTxRequest(executions, txContext);
  }
}

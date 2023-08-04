import { AztecAddress } from '@aztec/circuits.js';
import { FunctionCall, TxExecutionRequest } from '@aztec/types';

import { AccountImplementation, CreateTxRequestOpts } from './index.js';

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

  public createTxExecutionRequest(
    executions: FunctionCall[],
    opts: CreateTxRequestOpts = {},
  ): Promise<TxExecutionRequest> {
    const sender = opts.origin ?? this.getAddress();
    const impl = this.accounts.get(sender.toString());
    if (!impl) throw new Error(`No account implementation registered for ${sender}`);
    return impl.createTxExecutionRequest(executions, opts);
  }
}

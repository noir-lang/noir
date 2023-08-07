import { AztecAddress } from '@aztec/circuits.js';
import { FunctionCall, TxExecutionRequest } from '@aztec/types';

import { CreateTxRequestOpts, Entrypoint } from './index.js';

/**
 * An entrypoint that groups together multiple concrete entrypoints.
 * Delegates to the registered entrypoints based on the requested origin.
 */
export class EntrypointCollection implements Entrypoint {
  private entrypoints: Map<string, Entrypoint> = new Map();

  /**
   * Registers an entrypoint against an aztec address
   * @param addr - The aztec address agianst which to register the implementation.
   * @param impl - The entrypoint to be registered.
   */
  public registerAccount(addr: AztecAddress, impl: Entrypoint) {
    this.entrypoints.set(addr.toString(), impl);
  }

  public createTxExecutionRequest(
    executions: FunctionCall[],
    opts: CreateTxRequestOpts = {},
  ): Promise<TxExecutionRequest> {
    const defaultAccount = this.entrypoints.values().next().value as Entrypoint;
    const impl = opts.origin ? this.entrypoints.get(opts.origin.toString()) : defaultAccount;
    if (!impl) throw new Error(`No entrypoint registered for ${opts.origin}`);
    return impl.createTxExecutionRequest(executions, opts);
  }
}

import { AztecAddress } from '@aztec/circuits.js';
import { AztecRPC, Tx, TxExecutionRequest } from '@aztec/types';

import { SentTx } from './sent_tx.js';

/**
 * Represents options for calling a (constrained) function in a contract.
 * Allows the user to specify the sender address and nonce for a transaction.
 */
export interface SendMethodOptions {
  /**
   * Sender's address initiating the transaction.
   */
  origin?: AztecAddress;
}

/**
 * Base class for an interaction with a contract, be it a deployment, a function call, or a batch.
 * Implements the sequence create/simulate/send.
 */
export abstract class BaseContractInteraction {
  protected tx?: Tx;
  protected txRequest?: TxExecutionRequest;

  constructor(protected rpc: AztecRPC) {}

  /**
   * Create a transaction execution request ready to be simulated.
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns A transaction execution request.
   */
  public abstract create(options?: SendMethodOptions): Promise<TxExecutionRequest>;

  /**
   * Simulates a transaction execution request and returns a tx object ready to be sent.
   * @param options - optional arguments to be used in the creation of the transaction
   * @returns The resulting transaction
   */
  public async simulate(options: SendMethodOptions = {}): Promise<Tx> {
    const txRequest = this.txRequest ?? (await this.create(options));
    this.tx = await this.rpc.simulateTx(txRequest);
    return this.tx;
  }

  /**
   * Sends a transaction to the contract function with the specified options.
   * This function throws an error if called on an unconstrained function.
   * It creates and signs the transaction if necessary, and returns a SentTx instance,
   * which can be used to track the transaction status, receipt, and events.
   * @param options - An optional object containing 'from' property representing
   * the AztecAddress of the sender. If not provided, the default address is used.
   * @returns A SentTx instance for tracking the transaction status and information.
   */
  public send(options: SendMethodOptions = {}) {
    const promise = (async () => {
      const tx = this.tx ?? (await this.simulate(options));
      return this.rpc.sendTx(tx);
    })();

    return new SentTx(this.rpc, promise);
  }
}

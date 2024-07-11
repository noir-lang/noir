import { type Tx, type TxExecutionRequest } from '@aztec/circuit-types';
import { GasSettings } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

import { type Wallet } from '../account/wallet.js';
import { type ExecutionRequestInit, type FeeOptions } from '../entrypoint/entrypoint.js';
import { getGasLimits } from './get_gas_limits.js';
import { SentTx } from './sent_tx.js';

/**
 * Represents options for calling a (constrained) function in a contract.
 * Allows the user to specify the sender address and nonce for a transaction.
 */
export type SendMethodOptions = {
  /** Wether to skip the simulation of the public part of the transaction. */
  skipPublicSimulation?: boolean;
  /** The fee options for the transaction. */
  fee?: FeeOptions;
  /** Whether to run an initial simulation of the tx with high gas limit to figure out actual gas settings (will default to true later down the road). */
  estimateGas?: boolean;
};

/**
 * Base class for an interaction with a contract, be it a deployment, a function call, or a batch.
 * Implements the sequence create/simulate/send.
 */
export abstract class BaseContractInteraction {
  /**
   * The transaction execution result. Set by prove().
   * Made public for simple mocking.
   */
  public tx?: Tx;
  protected txRequest?: TxExecutionRequest;

  protected log = createDebugLogger('aztec:js:contract_interaction');

  constructor(protected wallet: Wallet) {}

  /**
   * Create a transaction execution request ready to be simulated.
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns A transaction execution request.
   */
  public abstract create(options?: SendMethodOptions): Promise<TxExecutionRequest>;

  /**
   * Proves a transaction execution request and returns a tx object ready to be sent.
   * @param options - optional arguments to be used in the creation of the transaction
   * @returns The resulting transaction
   */
  public async prove(options: SendMethodOptions = {}): Promise<Tx> {
    const txRequest = this.txRequest ?? (await this.create(options));
    this.tx = await this.wallet.proveTx(txRequest, !options.skipPublicSimulation);
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
      const tx = this.tx ?? (await this.prove(options));
      return this.wallet.sendTx(tx);
    })();

    return new SentTx(this.wallet, promise);
  }

  /**
   * Estimates gas for a given tx request and returns gas limits for it.
   * @param opts - Options.
   * @returns Gas limits.
   */
  public async estimateGas(
    opts?: Omit<SendMethodOptions, 'estimateGas' | 'skipPublicSimulation'>,
  ): Promise<Pick<GasSettings, 'gasLimits' | 'teardownGasLimits'>> {
    // REFACTOR: both `this.txRequest = undefined` below are horrible, we should not be caching stuff that doesn't need to be.
    // This also hints at a weird interface for create/request/estimate/send etc.

    // Ensure we don't accidentally use a version of tx request that has estimateGas set to true, leading to an infinite loop.
    this.txRequest = undefined;
    const txRequest = await this.create({ ...opts, estimateGas: false });
    // Ensure we don't accidentally cache a version of tx request that has estimateGas forcefully set to false.
    this.txRequest = undefined;

    const simulationResult = await this.wallet.simulateTx(txRequest, true);
    const { totalGas: gasLimits, teardownGas: teardownGasLimits } = getGasLimits(
      simulationResult,
      (opts?.fee?.gasSettings ?? GasSettings.default()).teardownGasLimits,
    );
    return { gasLimits, teardownGasLimits };
  }

  /**
   * Helper method to return fee options based on the user opts, estimating tx gas if needed.
   * @param request - Request to execute for this interaction.
   * @returns Fee options for the actual transaction.
   */
  protected async getFeeOptionsFromEstimatedGas(request: ExecutionRequestInit) {
    const fee = request.fee;
    if (fee) {
      const txRequest = await this.wallet.createTxExecutionRequest(request);
      const simulationResult = await this.wallet.simulateTx(txRequest, true);
      const { totalGas: gasLimits, teardownGas: teardownGasLimits } = getGasLimits(
        simulationResult,
        fee.gasSettings.teardownGasLimits,
      );
      this.log.debug(
        `Estimated gas limits for tx: DA=${gasLimits.daGas} L2=${gasLimits.l2Gas} teardownDA=${teardownGasLimits.daGas} teardownL2=${teardownGasLimits.l2Gas}`,
      );
      const gasSettings = GasSettings.default({ ...fee.gasSettings, gasLimits, teardownGasLimits });
      return { ...fee, gasSettings };
    }
    return fee;
  }
}

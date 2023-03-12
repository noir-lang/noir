import { retryUntil } from '../retry/index.js';
import { EthereumRpc } from './ethereum_rpc.js';
import { TxHash } from './tx_hash.js';
import { TransactionReceipt } from './types/index.js';

/**
 * Represents a transaction that has been sent.
 * It can be queried for its transaction hash, or its transaction receipt.
 * As it can be typed on a specific TxReceipt type, the receipt can have full type information regarding event logs.
 */
export interface SentTx<TxReceipt = TransactionReceipt> {
  getTxHash(): Promise<TxHash>;
  getReceipt(
    throwOnError?: boolean,
    numConfirmations?: number,
    timeout?: number,
    interval?: number,
  ): Promise<TxReceipt>;
}

/**
 * A standard implementation of SentTx.
 * This is returned by the `EthereumRpc.sendTransaction` call.
 * Can be extended to perform additional receipt handling in the `contract` module.
 */
export class SentTransaction implements SentTx {
  private receipt?: TransactionReceipt | null;

  constructor(protected ethRpc: EthereumRpc, protected txHashPromise: Promise<TxHash>) {}

  public async getTxHash(): Promise<TxHash> {
    return await this.txHashPromise;
  }

  public async getReceipt(
    throwOnError = true,
    numConfirmations = 1,
    timeout = 0,
    interval = 1,
  ): Promise<TransactionReceipt> {
    if (this.receipt) {
      return this.receipt;
    }

    const txHash = await this.getTxHash();
    const receipt = await waitForTxReceipt(txHash, this.ethRpc, numConfirmations, timeout, interval);
    return await this.handleReceipt(throwOnError, receipt);
  }

  protected handleReceipt(throwOnError = true, receipt: TransactionReceipt) {
    if (throwOnError && !receipt.status) {
      throw new Error('Receipt indicates transaction failed. Try a call() to determine cause of failure.');
    }
    return Promise.resolve(receipt);
  }
}

/**
 * A simple function to wait until a tx as a given number of confirmations, and return its receipt.
 */
export async function waitForTxReceipt(txHash: TxHash, eth: EthereumRpc, confirmations = 1, timeout = 0, interval = 1) {
  return await retryUntil(
    async () => {
      const blockNumber = await eth.blockNumber();
      const receipt = await eth.getTransactionReceipt(txHash);

      if (!receipt) {
        return;
      }

      if (receipt.status === false) {
        return receipt;
      }

      if (blockNumber - receipt.blockNumber + 1 >= confirmations) {
        return receipt;
      }
    },
    'waitForTxReceipt',
    timeout,
    interval,
  );
}

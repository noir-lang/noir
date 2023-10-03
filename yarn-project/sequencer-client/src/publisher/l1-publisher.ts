import { createDebugLogger } from '@aztec/foundation/log';
import { InterruptableSleep } from '@aztec/foundation/sleep';
import { ExtendedContractData, L2Block } from '@aztec/types';

import pick from 'lodash.pick';

import { L2BlockReceiver } from '../receiver.js';
import { PublisherConfig } from './config.js';
import { L1PublishStats } from './index.js';

/**
 * Stats for a sent transaction.
 */
export type TransactionStats = {
  /** Hash of the transaction. */
  transactionHash: string;
  /** Size in bytes of the tx calldata */
  calldataSize: number;
  /** Gas required to pay for the calldata inclusion (depends on size and number of zeros)  */
  calldataGas: number;
};

/**
 * Minimal information from a tx receipt returned by an L1PublisherTxSender.
 */
export type MinimalTransactionReceipt = {
  /** True if the tx was successful, false if reverted. */
  status: boolean;
  /** Hash of the transaction. */
  transactionHash: string;
  /** Effective gas used by the tx */
  gasUsed: bigint;
  /** Effective gas price paid by the tx */
  gasPrice: bigint;
};

/**
 * Pushes txs to the L1 chain and waits for their completion.
 */
export interface L1PublisherTxSender {
  /**
   * Sends a tx to the L1 rollup contract with a new L2 block. Returns once the tx has been mined.
   * @param encodedData - Serialized data for processing the new L2 block.
   * @returns The hash of the mined tx.
   */
  sendProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined>;

  /**
   * Sends a tx to the contract deployment emitter contract with contract deployment data such as bytecode. Returns once the tx has been mined.
   * @param l2BlockNum - Number of the L2 block that owns this encrypted logs.
   * @param l2BlockHash - The hash of the block corresponding to this data.
   * @param partialAddresses - The partial addresses of the deployed contract
   * @param publicKeys - The public keys of the deployed contract
   * @param newExtendedContractData - Data to publish.
   * @returns The hash of the mined tx.
   * @remarks Partial addresses, public keys and contract data has to be in the same order. Read more {@link https://docs.aztec.network/concepts/foundation/accounts/keys#addresses-partial-addresses-and-public-keys | here}.
   */
  sendEmitContractDeploymentTx(
    l2BlockNum: number,
    l2BlockHash: Buffer,
    newExtendedContractData: ExtendedContractData[],
  ): Promise<(string | undefined)[]>;

  /**
   * Returns a tx receipt if the tx has been mined.
   * @param txHash - Hash of the tx to look for.
   * @returns Undefined if the tx hasn't been mined yet, the receipt otherwise.
   */
  getTransactionReceipt(txHash: string): Promise<MinimalTransactionReceipt | undefined>;

  /**
   * Returns info on a tx by calling eth_getTransaction.
   * @param txHash - Hash of the tx to look for.
   */
  getTransactionStats(txHash: string): Promise<TransactionStats | undefined>;
}

/**
 * Encoded block data and proof ready to be pushed to the L1 contract.
 */
export type L1ProcessArgs = {
  /**
   * Root rollup proof for an L1 block.
   */
  proof: Buffer;
  /**
   * Serialized L2Block data.
   */
  inputs: Buffer;
};

/**
 * Helper function to filter out undefined items from an array.
 * Also asserts the resulting array is of type <T>.
 * @param item - An item from an array to check if undefined or not.
 * @returns True if the item is not undefined.
 */
function isNotUndefined<T>(item: T | undefined): item is T {
  return item !== undefined;
}

/**
 * Publishes L2 blocks to L1. This implementation does *not* retry a transaction in
 * the event of network congestion, but should work for local development.
 * - If sending (not mining) a tx fails, it retries indefinitely at 1-minute intervals.
 * - If the tx is not mined, keeps polling indefinitely at 1-second intervals.
 *
 * Adapted from https://github.com/AztecProtocol/aztec2-internal/blob/master/falafel/src/rollup_publisher.ts.
 */
export class L1Publisher implements L2BlockReceiver {
  private interruptableSleep = new InterruptableSleep();
  private sleepTimeMs: number;
  private interrupted = false;
  private log = createDebugLogger('aztec:sequencer:publisher');

  constructor(private txSender: L1PublisherTxSender, config?: PublisherConfig) {
    this.sleepTimeMs = config?.l1BlockPublishRetryIntervalMS ?? 60_000;
  }

  /**
   * Processes incoming L2 block data by publishing it to the L1 rollup contract.
   * @param l2BlockData - L2 block data to publish.
   * @returns True once the tx has been confirmed and is successful, false on revert or interrupt, blocks otherwise.
   */
  public async processL2Block(l2BlockData: L2Block): Promise<boolean> {
    const proof = Buffer.alloc(0);
    const txData = { proof, inputs: l2BlockData.encode() };

    while (!this.interrupted) {
      if (!(await this.checkFeeDistributorBalance())) {
        this.log(`Fee distributor ETH balance too low, awaiting top up...`);
        await this.sleepOrInterrupted();
        continue;
      }

      const txHash = await this.sendProcessTx(txData);
      if (!txHash) break;

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) break;

      // Tx was mined successfully
      if (receipt.status) {
        const tx = await this.txSender.getTransactionStats(txHash);
        const stats: L1PublishStats = {
          ...pick(receipt, 'gasPrice', 'gasUsed', 'transactionHash'),
          ...pick(tx!, 'calldataGas', 'calldataSize'),
          ...l2BlockData.getStats(),
          eventName: 'rollup-published-to-l1',
        };
        this.log.info(`Published L2 block to L1 rollup contract`, stats);
        return true;
      }

      // Check if someone else incremented the block number
      if (!(await this.checkNextL2BlockNum(l2BlockData.number))) {
        this.log('Publish failed. Contract changed underfoot.');
        break;
      }

      this.log(`Transaction status failed: ${receipt.transactionHash}`);
      await this.sleepOrInterrupted();
    }

    this.log('L2 block data syncing interrupted while processing blocks.');
    return false;
  }

  /**
   * Publishes new contract data to L1.
   * @param l2BlockNum - The L2 block number that the new contracts were deployed on.
   * @param l2BlockHash - The hash of the block corresponding to this data.
   * @param contractData - The new contract data to publish.
   * @returns True once the tx has been confirmed and is successful, false on revert or interrupt, blocks otherwise.
   */
  public async processNewContractData(l2BlockNum: number, l2BlockHash: Buffer, contractData: ExtendedContractData[]) {
    let _contractData: ExtendedContractData[] = [];
    while (!this.interrupted) {
      if (!(await this.checkFeeDistributorBalance())) {
        this.log(`Fee distributor ETH balance too low, awaiting top up...`);
        await this.sleepOrInterrupted();
        continue;
      }

      const arr = _contractData.length ? _contractData : contractData;
      const txHashes = await this.sendEmitNewContractDataTx(l2BlockNum, l2BlockHash, arr);
      if (!txHashes) break;
      // filter successful txs
      _contractData = arr.filter((_, i) => !!txHashes[i]);

      const receipts = await Promise.all(
        txHashes.filter(isNotUndefined).map(txHash => this.getTransactionReceipt(txHash)),
      );
      if (!receipts?.length) break;

      // ALL Txs were mined successfully
      if (receipts.length === contractData.length && receipts.every(r => r?.status)) return true;

      this.log(
        `Transaction status failed: ${receipts
          .filter(r => !r?.status)
          .map(r => r?.transactionHash)
          .join(',')}`,
      );
      await this.sleepOrInterrupted();
    }

    this.log('L2 block data syncing interrupted while processing contract data.');
    return false;
  }

  /**
   * Calling `interrupt` will cause any in progress call to `publishRollup` to return `false` asap.
   * Be warned, the call may return false even if the tx subsequently gets successfully mined.
   * In practice this shouldn't matter, as we'll only ever be calling `interrupt` when we know it's going to fail.
   * A call to `restart` is required before you can continue publishing.
   */
  public interrupt() {
    this.interrupted = true;
    this.interruptableSleep.interrupt();
  }

  /** Restarts the publisher after calling `interrupt`. */
  public restart() {
    this.interrupted = false;
  }

  // TODO: Check fee distributor has at least 0.5 ETH.
  // Related to https://github.com/AztecProtocol/aztec-packages/issues/1588
  // eslint-disable-next-line require-await
  private async checkFeeDistributorBalance(): Promise<boolean> {
    return true;
  }

  // TODO: Fail if blockchainStatus.nextBlockNum > thisBlockNum.
  // Related to https://github.com/AztecProtocol/aztec-packages/issues/1588
  private checkNextL2BlockNum(_thisBlockNum: number): Promise<boolean> {
    return Promise.resolve(true);
  }

  private async sendProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.sendProcessTx(encodedData);
      } catch (err) {
        this.log.error(`Rollup publish failed`, err);
        return undefined;
      }
    }
  }

  private async sendEmitNewContractDataTx(
    l2BlockNum: number,
    l2BlockHash: Buffer,
    newExtendedContractData: ExtendedContractData[],
  ) {
    while (!this.interrupted) {
      try {
        return await this.txSender.sendEmitContractDeploymentTx(l2BlockNum, l2BlockHash, newExtendedContractData);
      } catch (err) {
        this.log.error(`Error sending contract data to L1`, err);
        await this.sleepOrInterrupted();
      }
    }
  }

  private async getTransactionReceipt(txHash: string): Promise<MinimalTransactionReceipt | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.getTransactionReceipt(txHash);
      } catch (err) {
        //this.log.error(`Error getting tx receipt`, err);
        await this.sleepOrInterrupted();
      }
    }
  }

  protected async sleepOrInterrupted() {
    await this.interruptableSleep.sleep(this.sleepTimeMs);
  }
}

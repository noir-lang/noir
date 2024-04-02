import { type L2Block } from '@aztec/circuit-types';
import { type L1PublishStats } from '@aztec/circuit-types/stats';
import { createDebugLogger } from '@aztec/foundation/log';
import { InterruptibleSleep } from '@aztec/foundation/sleep';

import pick from 'lodash.pick';

import { type L2BlockReceiver } from '../receiver.js';
import { type PublisherConfig } from './config.js';

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
  /** Effective gas used by the tx. */
  gasUsed: bigint;
  /** Effective gas price paid by the tx. */
  gasPrice: bigint;
  /** Logs emitted in this tx. */
  logs: any[];
};

/**
 * Pushes txs to the L1 chain and waits for their completion.
 */
export interface L1PublisherTxSender {
  /**
   * Publishes tx effects to Availability Oracle.
   * @param encodedBody - Encoded block body.
   * @returns The hash of the mined tx.
   */
  sendPublishTx(encodedBody: Buffer): Promise<string | undefined>;

  /**
   * Sends a tx to the L1 rollup contract with a new L2 block. Returns once the tx has been mined.
   * @param encodedData - Serialized data for processing the new L2 block.
   * @returns The hash of the mined tx.
   */
  sendProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined>;

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

  /**
   * Returns the current archive root.
   * @returns The current archive root of the rollup contract.
   */
  getCurrentArchive(): Promise<Buffer>;

  /**
   * Checks if the transaction effects of the given block are available.
   * @param block - The block of which to check whether txs are available.
   * @returns True if the txs are available, false otherwise.
   */
  checkIfTxsAreAvailable(block: L2Block): Promise<boolean>;
}

/**
 * Encoded block and proof ready to be pushed to the L1 contract.
 */
export type L1ProcessArgs = {
  /** The L2 block header. */
  header: Buffer;
  /** A root of the archive tree after the L2 block is applied. */
  archive: Buffer;
  /** L2 block body. */
  body: Buffer;
  /** Root rollup proof of the L2 block. */
  proof: Buffer;
};

/**
 * Publishes L2 blocks to L1. This implementation does *not* retry a transaction in
 * the event of network congestion, but should work for local development.
 * - If sending (not mining) a tx fails, it retries indefinitely at 1-minute intervals.
 * - If the tx is not mined, keeps polling indefinitely at 1-second intervals.
 *
 * Adapted from https://github.com/AztecProtocol/aztec2-internal/blob/master/falafel/src/rollup_publisher.ts.
 */
export class L1Publisher implements L2BlockReceiver {
  private interruptibleSleep = new InterruptibleSleep();
  private sleepTimeMs: number;
  private interrupted = false;
  private log = createDebugLogger('aztec:sequencer:publisher');

  constructor(private txSender: L1PublisherTxSender, config?: PublisherConfig) {
    this.sleepTimeMs = config?.l1BlockPublishRetryIntervalMS ?? 60_000;
  }

  /**
   * Publishes L2 block on L1.
   * @param block - L2 block to publish.
   * @returns True once the tx has been confirmed and is successful, false on revert or interrupt, blocks otherwise.
   */
  public async processL2Block(block: L2Block): Promise<boolean> {
    // TODO(#4148) Remove this block number check, it's here because we don't currently have proper genesis state on the contract
    const lastArchive = block.header.lastArchive.root.toBuffer();
    if (block.number != 1 && !(await this.checkLastArchiveHash(lastArchive))) {
      this.log(`Detected different last archive prior to publishing a block, aborting publish...`);
      return false;
    }

    const encodedBody = block.body.toBuffer();

    // Publish block transaction effects
    while (!this.interrupted) {
      if (await this.txSender.checkIfTxsAreAvailable(block)) {
        this.log(`Transaction effects of a block ${block.number} already published.`);
        break;
      }

      const txHash = await this.sendPublishTx(encodedBody);
      if (!txHash) {
        return false;
      }

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) {
        return false;
      }

      if (receipt.status) {
        let txsEffectsHash;
        if (receipt.logs.length === 1) {
          // txsEffectsHash from IAvailabilityOracle.TxsPublished event
          txsEffectsHash = receipt.logs[0].data;
        } else {
          this.log(`Expected 1 log, got ${receipt.logs.length}`);
        }

        this.log.info(`Block txs effects published, txsEffectsHash: ${txsEffectsHash}`);
        break;
      }

      this.log(`AvailabilityOracle.publish tx status failed: ${receipt.transactionHash}`);
      await this.sleepOrInterrupted();
    }

    const processTxArgs = {
      header: block.header.toBuffer(),
      archive: block.archive.root.toBuffer(),
      body: encodedBody,
      proof: Buffer.alloc(0),
    };

    // Process block
    while (!this.interrupted) {
      const txHash = await this.sendProcessTx(processTxArgs);
      if (!txHash) {
        break;
      }

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) {
        break;
      }

      // Tx was mined successfully
      if (receipt.status) {
        const tx = await this.txSender.getTransactionStats(txHash);
        const stats: L1PublishStats = {
          ...pick(receipt, 'gasPrice', 'gasUsed', 'transactionHash'),
          ...pick(tx!, 'calldataGas', 'calldataSize'),
          ...block.getStats(),
          eventName: 'rollup-published-to-l1',
        };
        this.log.info(`Published L2 block to L1 rollup contract`, stats);
        return true;
      }

      // Check if someone else incremented the block number
      if (!(await this.checkLastArchiveHash(lastArchive))) {
        this.log('Publish failed. Detected different last archive hash.');
        break;
      }

      this.log(`Rollup.process tx status failed: ${receipt.transactionHash}`);
      await this.sleepOrInterrupted();
    }

    this.log('L2 block data syncing interrupted while processing blocks.');
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
    this.interruptibleSleep.interrupt();
  }

  /** Restarts the publisher after calling `interrupt`. */
  public restart() {
    this.interrupted = false;
  }

  /**
   * Verifies that the given value of last archive in a block header equals current archive of the rollup contract
   * @param lastArchive - The last archive of the block we wish to publish.
   * @returns Boolean indicating if the hashes are equal.
   */
  private async checkLastArchiveHash(lastArchive: Buffer): Promise<boolean> {
    const fromChain = await this.txSender.getCurrentArchive();
    const areSame = lastArchive.equals(fromChain);
    if (!areSame) {
      this.log(`CONTRACT ARCHIVE: ${fromChain.toString('hex')}`);
      this.log(`NEW BLOCK LAST ARCHIVE: ${lastArchive.toString('hex')}`);
    }
    return areSame;
  }

  private async sendPublishTx(encodedBody: Buffer): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.sendPublishTx(encodedBody);
      } catch (err) {
        this.log.error(`TxEffects publish failed`, err);
        return undefined;
      }
    }
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
    await this.interruptibleSleep.sleep(this.sleepTimeMs);
  }
}

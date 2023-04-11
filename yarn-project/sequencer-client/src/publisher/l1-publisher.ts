import { L2Block } from '@aztec/types';
import { createDebugLogger, InterruptableSleep } from '@aztec/foundation';
import { L2BlockReceiver } from '../receiver.js';
import { PublisherConfig } from './config.js';
import { UnverifiedData } from '@aztec/types';

/**
 * Component responsible of pushing the txs to the chain and waiting for completion.
 */
export interface L1PublisherTxSender {
  sendProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined>;
  sendEmitUnverifiedDataTx(l2BlockNum: number, unverifiedData: UnverifiedData): Promise<string | undefined>;
  getTransactionReceipt(txHash: string): Promise<{ status: boolean; transactionHash: string } | undefined>;
}

/**
 * Encoded block data and proof ready to be pushed to the L1 contract.
 */
export type L1ProcessArgs = {
  proof: Buffer;
  inputs: Buffer;
};

/**
 * Publishes L2 blocks and unverified data to L1. This implementation does *not* retry a transaction in
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
  private log = createDebugLogger('aztec:sequencer');

  constructor(private txSender: L1PublisherTxSender, config?: PublisherConfig) {
    this.sleepTimeMs = config?.retryIntervalMs ?? 60_000;
  }

  /**
   * Processes incoming L2 block data by publishing it to the L1 rollup contract.
   * @returns True once the tx has been confirmed and is successful, false on revert or interrupt, blocks otherwise.
   */
  public async processL2Block(l2BlockData: L2Block): Promise<boolean> {
    const proof = Buffer.alloc(0);
    const txData = { proof, inputs: l2BlockData.encode() };
    //this.log(`Publishing L2 block: ${l2BlockData.inspect()}`);

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
      if (receipt.status) return true;

      // Check if someone else incremented the block number
      if (!(await this.checkNextL2BlockNum(l2BlockData.number))) {
        this.log('Publish failed. Contract changed underfoot.');
        break;
      }

      this.log(`Transaction status failed: ${receipt.transactionHash}`);
      await this.sleepOrInterrupted();
    }

    this.log('L2 block interrupted interrupted.');
    return false;
  }

  /**
   * Publishes unverifiedData to L1.
   * @param l2BlockNum The L2 block number that the unverifiedData is associated with.
   * @param unverifiedData The unverifiedData to publish.
   * @returns True once the tx has been confirmed and is successful, false on revert or interrupt, blocks otherwise.
   */
  public async processUnverifiedData(l2BlockNum: number, unverifiedData: UnverifiedData): Promise<boolean> {
    while (!this.interrupted) {
      if (!(await this.checkFeeDistributorBalance())) {
        this.log(`Fee distributor ETH balance too low, awaiting top up...`);
        await this.sleepOrInterrupted();
        continue;
      }

      const txHash = await this.sendEmitUnverifiedDataTx(l2BlockNum, unverifiedData);
      if (!txHash) break;

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) break;

      // Tx was mined successfully
      if (receipt.status) return true;

      this.log(`Transaction status failed: ${receipt.transactionHash}`);
      await this.sleepOrInterrupted();
    }

    this.log('L2 block interrupted interrupted.');
    return false;
  }

  /**
   * Calling `interrupt` will cause any in progress call to `publishRollup` to return `false` asap.
   * Be warned, the call may return false even if the tx subsequently gets successfully mined.
   * In practice this shouldn't matter, as we'll only ever be calling `interrupt` when we know it's going to fail.
   * A call to `clearInterrupt` is required before you can continue publishing.
   */
  public interrupt() {
    this.interrupted = true;
    this.interruptableSleep.interrupt();
  }

  // TODO: Check fee distributor has at least 0.5 ETH.
  // eslint-disable-next-line require-await
  private async checkFeeDistributorBalance(): Promise<boolean> {
    return true;
  }

  // TODO: Fail if blockchainStatus.nextBlockNum > thisBlockNum.
  private checkNextL2BlockNum(_thisBlockNum: number): Promise<boolean> {
    return Promise.resolve(true);
  }

  private async sendProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.sendProcessTx(encodedData);
      } catch (err) {
        this.log(`Error sending tx to L1`, err);
        await this.sleepOrInterrupted();
      }
    }
  }

  private async sendEmitUnverifiedDataTx(
    l2BlockNum: number,
    unverifiedData: UnverifiedData,
  ): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.sendEmitUnverifiedDataTx(l2BlockNum, unverifiedData);
      } catch (err) {
        this.log(`Error sending tx to L1`, err);
        await this.sleepOrInterrupted();
      }
    }
  }

  private async getTransactionReceipt(
    txHash: string,
  ): Promise<{ status: boolean; transactionHash: string } | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.getTransactionReceipt(txHash);
      } catch (err) {
        this.log(`Error getting tx receipt`, err);
        await this.sleepOrInterrupted();
      }
    }
  }

  protected async sleepOrInterrupted() {
    await this.interruptableSleep.sleep(this.sleepTimeMs);
  }
}

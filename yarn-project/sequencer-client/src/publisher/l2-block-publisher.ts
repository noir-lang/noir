import { L2Block } from '@aztec/archiver';
import { L2BlockReceiver } from '../receiver.js';

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const MIN_FEE_DISTRIBUTOR_BALANCE = 5n * 10n ** 17n;

/**
 * Component responsible of pushing the txs to the chain and waiting for completion.
 */
export interface PublisherTxSender {
  sendTransaction(encodedData: L1ProcessRollupArgs): Promise<string | undefined>;
  getTransactionReceipt(txHash: string): Promise<{ status: boolean; transactionHash: string } | undefined>;
}

/**
 * Encoded block data and proof ready to be pushed to the L1 contract.
 */
export type L1ProcessRollupArgs = {
  proof: Buffer;
  inputs: Buffer;
};

/**
 * Publishes L2 blocks to the L1 rollup contracts. This implementation does *not* retry a transaction in
 * the event of network congestion.
 * - If sending (not mining) a tx fails, it retries indefinitely at 1-minute intervals.
 * - If the tx is not mined, keeps polling indefinitely at 1-second intervals.
 *
 * Adapted from https://github.com/AztecProtocol/aztec2-internal/blob/master/falafel/src/rollup_publisher.ts.
 */
export class L2BlockPublisher implements L2BlockReceiver {
  private interrupted = false;
  private interruptPromise = Promise.resolve();
  private interruptResolve = () => {};
  private sleepTimeMs: number;

  constructor(private txSender: PublisherTxSender, opts?: { sleepTimeMs?: number }) {
    this.interruptPromise = new Promise(resolve => (this.interruptResolve = resolve));
    this.sleepTimeMs = opts?.sleepTimeMs ?? 60_000;
  }

  /**
   * Processes incoming L2 block data by publishing it to the L1 rollup contract.
   * @returns True once the tx has been confirmed and is successful, false on revert or interrupt, blocks otherwise.
   */
  public async processL2Block(l2BlockData: L2Block): Promise<boolean> {
    const proof = Buffer.alloc(0);
    const txData = { proof, inputs: l2BlockData.encode() };

    while (!this.interrupted) {
      if (!(await this.checkFeeDistributorBalance())) {
        console.log(`Fee distributor ETH balance too low, awaiting top up...`);
        await this.sleepOrInterrupted();
        continue;
      }

      const txHash = await this.sendTransaction(txData);
      if (!txHash) break;

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) break;

      // Tx was mined successfully
      if (receipt.status) return true;

      // Check if someone else moved the block id
      if (!(await this.checkNextL2BlockId(l2BlockData.number))) {
        console.log('Publish failed. Contract changed underfoot.');
        break;
      }

      console.log(`Transaction status failed: ${receipt.transactionHash}`);
      await this.sleepOrInterrupted();
    }

    console.log('Publish rollup interrupted.');
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
    this.interruptResolve();
  }

  // TODO: Check fee distributor has at least 0.5 ETH.
  // eslint-disable-next-line require-await
  private async checkFeeDistributorBalance(): Promise<boolean> {
    return true;
  }

  // TODO: Fail if blockchainStatus.nextRollupId > thisBlockId.
  // eslint-disable-next-line require-await, @typescript-eslint/no-unused-vars
  private async checkNextL2BlockId(thisBlockId: number): Promise<boolean> {
    return true;
  }

  private async sendTransaction(encodedData: L1ProcessRollupArgs): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.sendTransaction(encodedData);
      } catch (err) {
        console.log(`Error sending tx to L1`, err);
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
        console.log(`Error getting tx receipt`, err);
        await this.sleepOrInterrupted();
      }
    }
  }

  protected async sleepOrInterrupted() {
    const ms = this.sleepTimeMs;
    await Promise.race([new Promise(resolve => setTimeout(resolve, ms)), this.interruptPromise]);
  }
}

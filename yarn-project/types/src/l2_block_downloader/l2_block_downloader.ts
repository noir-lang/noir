import { MemoryFifo, Semaphore, SerialQueue } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { InterruptableSleep } from '@aztec/foundation/sleep';

import { INITIAL_L2_BLOCK_NUM, L2Block, L2BlockSource } from '../index.js';

const log = createDebugLogger('aztec:l2_block_downloader');

/**
 * Downloads L2 blocks from a L2BlockSource.
 * The blocks are stored in a queue and can be retrieved using the getBlocks method.
 * The queue size is limited by the maxQueueSize parameter.
 * The downloader will pause when the queue is full or when the L2BlockSource is out of blocks.
 */
export class L2BlockDownloader {
  private runningPromise?: Promise<void>;
  private running = false;
  private from = 0;
  private interruptableSleep = new InterruptableSleep();
  private semaphore: Semaphore;
  private jobQueue = new SerialQueue();
  private blockQueue = new MemoryFifo<L2Block[]>();

  constructor(private l2BlockSource: L2BlockSource, maxQueueSize: number, private pollIntervalMS = 10000) {
    this.semaphore = new Semaphore(maxQueueSize);
  }

  /**
   * Starts the downloader.
   * @param from - The block number to start downloading from. Defaults to INITIAL_L2_BLOCK_NUM.
   */
  public start(from = INITIAL_L2_BLOCK_NUM) {
    if (this.running) {
      this.interruptableSleep.interrupt();
      return;
    }
    this.from = from;
    this.running = true;

    const fn = async () => {
      while (this.running) {
        try {
          await this.jobQueue.put(() => this.collectBlocks());
          await this.interruptableSleep.sleep(this.pollIntervalMS);
        } catch (err) {
          log.error(err);
          await this.interruptableSleep.sleep(this.pollIntervalMS);
        }
      }
    };
    this.jobQueue.start();
    this.runningPromise = fn();
  }

  /**
   * Repeatedly queries the block source and adds the received blocks to the block queue.
   * Stops when no further blocks are received.
   * @returns The total number of blocks added to the block queue.
   */
  private async collectBlocks() {
    let totalBlocks = 0;
    while (true) {
      const blocks = await this.l2BlockSource.getL2Blocks(this.from, 10);
      if (!blocks.length) {
        return totalBlocks;
      }
      await this.semaphore.acquire();
      this.blockQueue.put(blocks);
      this.from += blocks.length;
      totalBlocks += blocks.length;
    }
  }

  /**
   * Stops the downloader.
   */
  public async stop() {
    this.running = false;
    this.interruptableSleep.interrupt();
    await this.jobQueue.cancel();
    this.blockQueue.cancel();
    await this.runningPromise;
  }

  /**
   * Gets the next batch of blocks from the queue.
   * @param timeout - optional timeout value to prevent permanent blocking
   * @returns The next batch of blocks from the queue.
   */
  public async getL2Blocks(timeout?: number) {
    try {
      const blocks = await this.blockQueue.get(timeout);
      if (!blocks) {
        return [];
      }
      this.semaphore.release();
      return blocks;
    } catch (err) {
      // nothing to do
      return [];
    }
  }

  /**
   * Forces an immediate request for blocks.
   * @returns A promise that fulfills once the poll is complete
   */
  public pollImmediate(): Promise<number> {
    return this.jobQueue.put(() => this.collectBlocks());
  }
}

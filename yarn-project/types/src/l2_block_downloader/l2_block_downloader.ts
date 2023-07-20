import { MemoryFifo, Semaphore } from '@aztec/foundation/fifo';
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
  private queue = new MemoryFifo<L2Block[]>();

  constructor(private l2BlockSource: L2BlockSource, maxQueueSize: number, private pollIntervalMS = 10000) {
    this.semaphore = new Semaphore(maxQueueSize);
  }

  /**
   * Starts the downloader.
   * @param from - The block number to start downloading from. Defaults to INITIAL_L2_BLOCK_NUM.
   */
  public start(from = INITIAL_L2_BLOCK_NUM) {
    this.from = from;

    if (this.running) {
      this.interruptableSleep.interrupt();
      return;
    }

    this.running = true;

    const fn = async () => {
      while (this.running) {
        try {
          const blocks = await this.l2BlockSource.getL2Blocks(this.from, 10);

          if (!blocks.length) {
            await this.interruptableSleep.sleep(this.pollIntervalMS);
            continue;
          }

          // Blocks if there are maxQueueSize results in the queue, until released after the callback.
          await this.semaphore.acquire();
          this.queue.put(blocks);
          this.from += blocks.length;
        } catch (err) {
          log(err);
          await this.interruptableSleep.sleep(this.pollIntervalMS);
        }
      }
    };

    this.runningPromise = fn();
  }

  /**
   * Stops the downloader.
   */
  public async stop() {
    this.running = false;
    this.interruptableSleep.interrupt();
    this.queue.cancel();
    await this.runningPromise;
  }

  /**
   * Gets the next batch of blocks from the queue.
   * @returns The next batch of blocks from the queue.
   */
  public async getL2Blocks() {
    const blocks = await this.queue.get();
    if (!blocks) {
      return [];
    }
    this.semaphore.release();
    return blocks;
  }
}

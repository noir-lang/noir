import { INITIAL_L2_BLOCK_NUM } from '@aztec/circuits.js/constants';
import { createDebugLogger } from '@aztec/foundation/log';
import { FifoMemoryQueue, Semaphore, SerialQueue } from '@aztec/foundation/queue';
import { InterruptibleSleep } from '@aztec/foundation/sleep';

import { type L2Block } from '../l2_block.js';
import { type L2BlockSource } from '../l2_block_source.js';

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
  private interruptibleSleep = new InterruptibleSleep();
  private readonly semaphore: Semaphore;
  private readonly jobQueue = new SerialQueue();
  private readonly blockQueue = new FifoMemoryQueue<L2Block[]>();
  private readonly proven: boolean;
  private readonly pollIntervalMS: number;

  constructor(
    private l2BlockSource: L2BlockSource,
    opts: {
      maxQueueSize: number;
      proven?: boolean;
      pollIntervalMS?: number;
    },
  ) {
    this.pollIntervalMS = opts.pollIntervalMS ?? 1000;
    this.proven = opts.proven ?? false;
    this.semaphore = new Semaphore(opts.maxQueueSize);
  }

  /**
   * Starts the downloader.
   * @param from - The block number to start downloading from. Defaults to INITIAL_L2_BLOCK_NUM.
   */
  public start(from = INITIAL_L2_BLOCK_NUM) {
    if (this.running) {
      this.interruptibleSleep.interrupt();
      return;
    }
    this.from = from;
    this.running = true;

    const fn = async () => {
      while (this.running) {
        try {
          await this.jobQueue.put(() => this.collectBlocks());
          await this.interruptibleSleep.sleep(this.pollIntervalMS);
        } catch (err) {
          log.error(`Error downloading L2 block`, err);
          await this.interruptibleSleep.sleep(this.pollIntervalMS);
        }
      }
    };
    this.jobQueue.start();
    this.runningPromise = fn();
  }

  /**
   * Repeatedly queries the block source and adds the received blocks to the block queue.
   * Stops when no further blocks are received.
   * @param targetBlockNumber - Optional block number to stop at.
   * @param proven - Optional override of the default "proven" setting.
   * @returns The total number of blocks added to the block queue.
   */
  private async collectBlocks(targetBlockNumber?: number, onlyProven?: boolean) {
    let totalBlocks = 0;
    while (true) {
      // If we have a target and have reached it, return
      if (targetBlockNumber !== undefined && this.from > targetBlockNumber) {
        log.verbose(`Reached target block number ${targetBlockNumber}`);
        return totalBlocks;
      }

      // If we have a target, then request at most the number of blocks to get to it
      const limit = targetBlockNumber !== undefined ? Math.min(targetBlockNumber - this.from + 1, 10) : 10;
      const proven = onlyProven === undefined ? this.proven : onlyProven;

      // Hit the archiver for blocks
      const blocks = await this.l2BlockSource.getBlocks(this.from, limit, proven);

      // If there are no more blocks, return
      if (!blocks.length) {
        return totalBlocks;
      }

      log.verbose(
        `Received ${blocks.length} blocks from archiver after querying from ${this.from} limit ${limit} (proven ${proven})`,
      );

      // Push new blocks into the queue and loop
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
    this.interruptibleSleep.interrupt();
    await this.jobQueue.cancel();
    this.blockQueue.cancel();
    await this.runningPromise;
  }

  /**
   * Gets the next batch of blocks from the queue.
   * @param timeout - optional timeout value to prevent permanent blocking
   * @returns The next batch of blocks from the queue.
   */
  public async getBlocks(timeout?: number): Promise<L2Block[]> {
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
   * Repeatedly queries the block source and adds the received blocks to the block queue.
   * Stops when no further blocks are received.
   * @param targetBlockNumber - Optional block number to stop at.
   * @param proven - Optional override of the default "proven" setting.
   * @returns A promise that fulfills once the poll is complete
   */
  public pollImmediate(targetBlockNumber?: number, onlyProven?: boolean): Promise<number> {
    return this.jobQueue.put(() => this.collectBlocks(targetBlockNumber, onlyProven));
  }
}

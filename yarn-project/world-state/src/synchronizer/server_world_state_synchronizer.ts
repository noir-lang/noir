import { SerialQueue } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import { L2Block, L2BlockDownloader, L2BlockSource } from '@aztec/types';

import { LevelUp } from 'levelup';

import { HandleL2BlockResult, MerkleTreeOperations, MerkleTrees } from '../index.js';
import { MerkleTreeOperationsFacade } from '../merkle-tree/merkle_tree_operations_facade.js';
import { WorldStateConfig } from './config.js';
import { WorldStateRunningState, WorldStateStatus, WorldStateSynchronizer } from './world_state_synchronizer.js';

const DB_KEY_BLOCK_NUMBER = 'latestBlockNumber';

/**
 * Synchronizes the world state with the L2 blocks from a L2BlockSource.
 * The synchronizer will download the L2 blocks from the L2BlockSource and insert the new commitments into the merkle
 * tree.
 */
export class ServerWorldStateSynchronizer implements WorldStateSynchronizer {
  private currentL2BlockNum = 0;
  private latestBlockNumberAtStart = 0;

  private l2BlockDownloader: L2BlockDownloader;
  private syncPromise: Promise<void> = Promise.resolve();
  private syncResolve?: () => void = undefined;
  private jobQueue = new SerialQueue();
  private stopping = false;
  private runningPromise: Promise<void> = Promise.resolve();
  private currentState: WorldStateRunningState = WorldStateRunningState.IDLE;

  private constructor(
    private db: LevelUp,
    private merkleTreeDb: MerkleTrees,
    private l2BlockSource: L2BlockSource,
    config: WorldStateConfig,
    private log = createDebugLogger('aztec:world_state'),
  ) {
    this.l2BlockDownloader = new L2BlockDownloader(
      l2BlockSource,
      config.l2QueueSize,
      config.worldStateBlockCheckIntervalMS,
    );
  }

  public getLatest(): MerkleTreeOperations {
    return new MerkleTreeOperationsFacade(this.merkleTreeDb, true);
  }

  public getCommitted(): MerkleTreeOperations {
    return new MerkleTreeOperationsFacade(this.merkleTreeDb, false);
  }

  public static async new(
    db: LevelUp,
    merkleTreeDb: MerkleTrees,
    l2BlockSource: L2BlockSource,
    config: WorldStateConfig,
    log = createDebugLogger('aztec:world_state'),
  ) {
    const server = new ServerWorldStateSynchronizer(db, merkleTreeDb, l2BlockSource, config, log);
    await server.#init();
    return server;
  }

  async #init() {
    await this.restoreCurrentL2BlockNumber();
  }

  public async start() {
    if (this.currentState === WorldStateRunningState.STOPPED) {
      throw new Error('Synchronizer already stopped');
    }
    if (this.currentState !== WorldStateRunningState.IDLE) {
      return this.syncPromise;
    }

    // get the current latest block number
    this.latestBlockNumberAtStart = await this.l2BlockSource.getBlockNumber();

    const blockToDownloadFrom = this.currentL2BlockNum + 1;

    // if there are blocks to be retrieved, go to a synching state
    if (blockToDownloadFrom <= this.latestBlockNumberAtStart) {
      this.setCurrentState(WorldStateRunningState.SYNCHING);
      this.syncPromise = new Promise(resolve => {
        this.syncResolve = resolve;
      });
      this.log(`Starting sync from ${blockToDownloadFrom}, latest block ${this.latestBlockNumberAtStart}`);
    } else {
      // if no blocks to be retrieved, go straight to running
      this.setCurrentState(WorldStateRunningState.RUNNING);
      this.syncPromise = Promise.resolve();
      this.log(`Next block ${blockToDownloadFrom} already beyond latest block at ${this.latestBlockNumberAtStart}`);
    }

    // start looking for further blocks
    const blockProcess = async () => {
      while (!this.stopping) {
        await this.jobQueue.put(() => this.collectAndProcessBlocks());
      }
    };
    this.jobQueue.start();
    this.runningPromise = blockProcess();
    this.l2BlockDownloader.start(blockToDownloadFrom);
    this.log(`Started block downloader from block ${blockToDownloadFrom}`);
    return this.syncPromise;
  }

  public async stop() {
    this.log('Stopping world state...');
    this.stopping = true;
    await this.l2BlockDownloader.stop();
    await this.jobQueue.cancel();
    await this.merkleTreeDb.stop();
    await this.runningPromise;
    await this.commitCurrentL2BlockNumber();
    this.setCurrentState(WorldStateRunningState.STOPPED);
  }

  public status(): Promise<WorldStateStatus> {
    const status = {
      syncedToL2Block: this.currentL2BlockNum,
      state: this.currentState,
    } as WorldStateStatus;
    return Promise.resolve(status);
  }

  /**
   * Forces an immediate sync
   * @param minBlockNumber - The minimum block number that we must sync to
   * @returns A promise that resolves once the sync has completed.
   */
  public async syncImmediate(minBlockNumber?: number): Promise<void> {
    if (this.currentState !== WorldStateRunningState.RUNNING) {
      throw new Error(`World State is not running, unable to perform sync`);
    }
    // If we have been given a block number to sync to and we have reached that number
    // then return.
    if (minBlockNumber !== undefined && minBlockNumber <= this.currentL2BlockNum) {
      return;
    }
    const blockToSyncTo = minBlockNumber === undefined ? 'latest' : `${minBlockNumber}`;
    this.log(`World State at block ${this.currentL2BlockNum}, told to sync to block ${blockToSyncTo}...`);
    // ensure any outstanding block updates are completed first.
    await this.jobQueue.syncPoint();
    while (true) {
      // Check the block number again
      if (minBlockNumber !== undefined && minBlockNumber <= this.currentL2BlockNum) {
        return;
      }
      // Poll for more blocks
      const numBlocks = await this.l2BlockDownloader.pollImmediate();
      this.log(`Block download immediate poll yielded ${numBlocks} blocks`);
      if (numBlocks) {
        // More blocks were received, process them and go round again
        await this.jobQueue.put(() => this.collectAndProcessBlocks());
        continue;
      }
      // No blocks are available, if we have been given a block number then we can't achieve it
      if (minBlockNumber !== undefined) {
        throw new Error(
          `Unable to sync to block number ${minBlockNumber}, currently synced to block ${this.currentL2BlockNum}`,
        );
      }
      return;
    }
  }

  /**
   * Checks for the availability of new blocks and processes them.
   */
  private async collectAndProcessBlocks() {
    // This request for blocks will timeout after 1 second if no blocks are received
    const blocks = await this.l2BlockDownloader.getL2Blocks(1);
    await this.handleL2Blocks(blocks);
    await this.commitCurrentL2BlockNumber();
  }

  /**
   * Handles a list of L2 blocks (i.e. Inserts the new commitments into the merkle tree).
   * @param l2Blocks - The L2 blocks to handle.
   * @returns Whether the block handled was produced by this same node.
   */
  private async handleL2Blocks(l2Blocks: L2Block[]) {
    for (const l2Block of l2Blocks) {
      const [duration, result] = await elapsed(() => this.handleL2Block(l2Block));
      this.log(`Handled new L2 block`, {
        eventName: 'l2-block-handled',
        duration,
        isBlockOurs: result.isBlockOurs,
        ...l2Block.getStats(),
      });
    }
  }

  /**
   * Handles a single L2 block (i.e. Inserts the new commitments into the merkle tree).
   * @param l2Block - The L2 block to handle.
   */
  private async handleL2Block(l2Block: L2Block): Promise<HandleL2BlockResult> {
    const result = await this.merkleTreeDb.handleL2Block(l2Block);
    this.currentL2BlockNum = l2Block.number;
    if (
      this.currentState === WorldStateRunningState.SYNCHING &&
      this.currentL2BlockNum >= this.latestBlockNumberAtStart
    ) {
      this.setCurrentState(WorldStateRunningState.RUNNING);
      if (this.syncResolve !== undefined) {
        this.syncResolve();
      }
    }
    return result;
  }

  /**
   * Method to set the value of the current state.
   * @param newState - New state value.
   */
  private setCurrentState(newState: WorldStateRunningState) {
    this.currentState = newState;
    this.log(`Moved to state ${WorldStateRunningState[this.currentState]}`);
  }

  private async commitCurrentL2BlockNumber() {
    const hex = this.currentL2BlockNum.toString(16);
    const encoded = Buffer.from(hex.length % 2 === 1 ? '0' + hex : hex, 'hex');

    await this.db.put(DB_KEY_BLOCK_NUMBER, encoded);
  }

  private async restoreCurrentL2BlockNumber() {
    try {
      const encoded: Buffer = await this.db.get(DB_KEY_BLOCK_NUMBER);
      this.currentL2BlockNum = parseInt(encoded.toString('hex'), 16);
      this.log.debug(`Restored current L2 block number ${this.currentL2BlockNum} from db`);
    } catch (err) {
      this.log.debug('No current L2 block number found in db, starting from 0');
      this.currentL2BlockNum = 0;
    }
  }
}

import { L2Block, L2BlockDownloader, L2BlockSource } from '@aztec/circuit-types';
import { L2BlockHandledStats } from '@aztec/circuit-types/stats';
import { SerialQueue } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import { AztecKVStore, AztecSingleton } from '@aztec/kv-store';

import { HandleL2BlockResult, MerkleTreeOperations, MerkleTrees } from '../world-state-db/index.js';
import { MerkleTreeOperationsFacade } from '../world-state-db/merkle_tree_operations_facade.js';
import { MerkleTreeSnapshotOperationsFacade } from '../world-state-db/merkle_tree_snapshot_operations_facade.js';
import { WorldStateConfig } from './config.js';
import { WorldStateRunningState, WorldStateStatus, WorldStateSynchronizer } from './world_state_synchronizer.js';

/**
 * Synchronizes the world state with the L2 blocks from a L2BlockSource.
 * The synchronizer will download the L2 blocks from the L2BlockSource and insert the new commitments into the merkle
 * tree.
 */
export class ServerWorldStateSynchronizer implements WorldStateSynchronizer {
  private latestBlockNumberAtStart = 0;

  private l2BlockDownloader: L2BlockDownloader;
  private syncPromise: Promise<void> = Promise.resolve();
  private syncResolve?: () => void = undefined;
  private jobQueue = new SerialQueue();
  private stopping = false;
  private runningPromise: Promise<void> = Promise.resolve();
  private currentState: WorldStateRunningState = WorldStateRunningState.IDLE;
  private blockNumber: AztecSingleton<number>;

  constructor(
    store: AztecKVStore,
    private merkleTreeDb: MerkleTrees,
    private l2BlockSource: L2BlockSource,
    config: WorldStateConfig,
    private log = createDebugLogger('aztec:world_state'),
  ) {
    this.blockNumber = store.openSingleton('world_state_synch_last_block_number');
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

  public getSnapshot(blockNumber: number): MerkleTreeOperations {
    return new MerkleTreeSnapshotOperationsFacade(this.merkleTreeDb, blockNumber);
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
    this.log('Cancelling job queue...');
    await this.jobQueue.cancel();
    this.log('Stopping Merkle trees');
    await this.merkleTreeDb.stop();
    this.log('Awaiting promise');
    await this.runningPromise;
    this.setCurrentState(WorldStateRunningState.STOPPED);
  }

  private get currentL2BlockNum(): number {
    return this.blockNumber.get() ?? 0;
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
   * @returns A promise that resolves with the block number the world state was synced to
   */
  public async syncImmediate(minBlockNumber?: number): Promise<number> {
    if (this.currentState !== WorldStateRunningState.RUNNING) {
      throw new Error(`World State is not running, unable to perform sync`);
    }
    // If we have been given a block number to sync to and we have reached that number
    // then return.
    if (minBlockNumber !== undefined && minBlockNumber <= this.currentL2BlockNum) {
      return this.currentL2BlockNum;
    }
    const blockToSyncTo = minBlockNumber === undefined ? 'latest' : `${minBlockNumber}`;
    this.log(`World State at block ${this.currentL2BlockNum}, told to sync to block ${blockToSyncTo}...`);
    // ensure any outstanding block updates are completed first.
    await this.jobQueue.syncPoint();
    while (true) {
      // Check the block number again
      if (minBlockNumber !== undefined && minBlockNumber <= this.currentL2BlockNum) {
        return this.currentL2BlockNum;
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
      return this.currentL2BlockNum;
    }
  }

  /**
   * Checks for the availability of new blocks and processes them.
   */
  private async collectAndProcessBlocks() {
    // This request for blocks will timeout after 1 second if no blocks are received
    const blocks = await this.l2BlockDownloader.getBlocks(1);
    await this.handleL2Blocks(blocks);
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
      } satisfies L2BlockHandledStats);
    }
  }

  /**
   * Handles a single L2 block (i.e. Inserts the new commitments into the merkle tree).
   * @param l2Block - The L2 block to handle.
   */
  private async handleL2Block(l2Block: L2Block): Promise<HandleL2BlockResult> {
    const result = await this.merkleTreeDb.handleL2Block(l2Block);
    await this.blockNumber.set(l2Block.number);

    if (this.currentState === WorldStateRunningState.SYNCHING && l2Block.number >= this.latestBlockNumberAtStart) {
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
}

import { type L1ToL2MessageSource, type L2Block, L2BlockDownloader, type L2BlockSource } from '@aztec/circuit-types';
import { type L2BlockHandledStats } from '@aztec/circuit-types/stats';
import { L1_TO_L2_MSG_SUBTREE_HEIGHT } from '@aztec/circuits.js/constants';
import { Fr } from '@aztec/foundation/fields';
import { SerialQueue } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import { type AztecKVStore, type AztecSingleton } from '@aztec/kv-store';
import { openTmpStore } from '@aztec/kv-store/utils';
import { SHA256Trunc, StandardTree } from '@aztec/merkle-tree';

import {
  type HandleL2BlockAndMessagesResult,
  type MerkleTreeOperations,
  type MerkleTrees,
} from '../world-state-db/index.js';
import { MerkleTreeOperationsFacade } from '../world-state-db/merkle_tree_operations_facade.js';
import { MerkleTreeSnapshotOperationsFacade } from '../world-state-db/merkle_tree_snapshot_operations_facade.js';
import { type WorldStateConfig } from './config.js';
import {
  WorldStateRunningState,
  type WorldStateStatus,
  type WorldStateSynchronizer,
} from './world_state_synchronizer.js';

/**
 * Synchronizes the world state with the L2 blocks from a L2BlockSource.
 * The synchronizer will download the L2 blocks from the L2BlockSource and insert the new note hashes into the merkle
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
    private l2BlockSource: L2BlockSource & L1ToL2MessageSource,
    config: WorldStateConfig,
    private log = createDebugLogger('aztec:world_state'),
  ) {
    this.blockNumber = store.openSingleton('world_state_synch_last_block_number');
    this.l2BlockDownloader = new L2BlockDownloader(l2BlockSource, {
      maxQueueSize: config.l2QueueSize,
      pollIntervalMS: config.worldStateBlockCheckIntervalMS,
      proven: config.worldStateProvenBlocksOnly,
    });
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
      this.log.info(`Starting sync from ${blockToDownloadFrom}, latest block ${this.latestBlockNumberAtStart}`);
    } else {
      // if no blocks to be retrieved, go straight to running
      this.setCurrentState(WorldStateRunningState.RUNNING);
      this.syncPromise = Promise.resolve();
      this.log.debug(
        `Next block ${blockToDownloadFrom} already beyond latest block at ${this.latestBlockNumberAtStart}`,
      );
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
    this.log.info(`Started block downloader from block ${blockToDownloadFrom}`);
    return this.syncPromise;
  }

  public async stop() {
    this.log.debug('Stopping world state...');
    this.stopping = true;
    await this.l2BlockDownloader.stop();
    this.log.debug('Cancelling job queue...');
    await this.jobQueue.cancel();
    this.log.debug('Stopping Merkle trees');
    await this.merkleTreeDb.stop();
    this.log.debug('Awaiting promise');
    await this.runningPromise;
    this.setCurrentState(WorldStateRunningState.STOPPED);
    this.log.info(`Stopped`);
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
    this.log.debug(`World State at block ${this.currentL2BlockNum}, told to sync to block ${blockToSyncTo}...`);
    // ensure any outstanding block updates are completed first.
    await this.jobQueue.syncPoint();
    while (true) {
      // Check the block number again
      if (minBlockNumber !== undefined && minBlockNumber <= this.currentL2BlockNum) {
        return this.currentL2BlockNum;
      }
      // Poll for more blocks
      const numBlocks = await this.l2BlockDownloader.pollImmediate();
      this.log.debug(`Block download immediate poll yielded ${numBlocks} blocks`);
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
    const messagePromises = blocks.map(block => this.l2BlockSource.getL1ToL2Messages(BigInt(block.number)));
    const l1ToL2Messages: Fr[][] = await Promise.all(messagePromises);

    await this.handleL2BlocksAndMessages(blocks, l1ToL2Messages);
  }

  /**
   * Handles a list of L2 blocks (i.e. Inserts the new note hashes into the merkle tree).
   * @param l2Blocks - The L2 blocks to handle.
   * @param l1ToL2Messages - The L1 to L2 messages for each block.
   * @returns Whether the block handled was produced by this same node.
   */
  private async handleL2BlocksAndMessages(l2Blocks: L2Block[], l1ToL2Messages: Fr[][]) {
    for (let i = 0; i < l2Blocks.length; i++) {
      const [duration, result] = await elapsed(() => this.handleL2BlockAndMessages(l2Blocks[i], l1ToL2Messages[i]));
      this.log.verbose(`Handled new L2 block`, {
        eventName: 'l2-block-handled',
        duration,
        isBlockOurs: result.isBlockOurs,
        ...l2Blocks[i].getStats(),
      } satisfies L2BlockHandledStats);
    }
  }

  /**
   * Handles a single L2 block (i.e. Inserts the new note hashes into the merkle tree).
   * @param l2Block - The L2 block to handle.
   * @param l1ToL2Messages - The L1 to L2 messages for the block.
   * @returns Whether the block handled was produced by this same node.
   */
  private async handleL2BlockAndMessages(
    l2Block: L2Block,
    l1ToL2Messages: Fr[],
  ): Promise<HandleL2BlockAndMessagesResult> {
    // First we check that the L1 to L2 messages hash to the block inHash.
    // Note that we cannot optimize this check by checking the root of the subtree after inserting the messages
    // to the real L1_TO_L2_MESSAGE_TREE (like we do in merkleTreeDb.handleL2BlockAndMessages(...)) because that
    // tree uses pedersen and we don't have access to the converted root.
    await this.#verifyMessagesHashToInHash(l1ToL2Messages, l2Block.header.contentCommitment.inHash);

    // If the above check succeeds, we can proceed to handle the block.
    const result = await this.merkleTreeDb.handleL2BlockAndMessages(l2Block, l1ToL2Messages);
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
    this.log.debug(`Moved to state ${WorldStateRunningState[this.currentState]}`);
  }

  /**
   * Verifies that the L1 to L2 messages hash to the block inHash.
   * @param l1ToL2Messages - The L1 to L2 messages for the block.
   * @param inHash - The inHash of the block.
   * @throws If the L1 to L2 messages do not hash to the block inHash.
   */
  async #verifyMessagesHashToInHash(l1ToL2Messages: Fr[], inHash: Buffer) {
    const tree = new StandardTree(
      openTmpStore(true),
      new SHA256Trunc(),
      'temp_in_hash_check',
      L1_TO_L2_MSG_SUBTREE_HEIGHT,
      0n,
      Fr,
    );
    await tree.appendLeaves(l1ToL2Messages);

    if (!tree.getRoot(true).equals(inHash)) {
      throw new Error('Obtained L1 to L2 messages failed to be hashed to the block inHash');
    }
  }
}

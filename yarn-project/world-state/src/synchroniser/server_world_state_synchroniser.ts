import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { L2Block, L2BlockDownloader, L2BlockSource } from '@aztec/types';

import { MerkleTreeDb, MerkleTreeOperations, computeGlobalVariablesHash } from '../index.js';
import { MerkleTreeOperationsFacade } from '../merkle-tree/merkle_tree_operations_facade.js';
import { getConfigEnvVars } from './config.js';
import { WorldStateRunningState, WorldStateStatus, WorldStateSynchroniser } from './world_state_synchroniser.js';

/**
 * Synchronises the world state with the L2 blocks from a L2BlockSource.
 * The synchroniser will download the L2 blocks from the L2BlockSource and insert the new commitments into the merkle
 * tree.
 */
export class ServerWorldStateSynchroniser implements WorldStateSynchroniser {
  private currentL2BlockNum = 0;
  private latestBlockNumberAtStart = 0;

  private l2BlockDownloader: L2BlockDownloader;
  private syncPromise: Promise<void> = Promise.resolve();
  private syncResolve?: () => void = undefined;
  private stopping = false;
  private runningPromise: Promise<void> = Promise.resolve();
  private currentState: WorldStateRunningState = WorldStateRunningState.IDLE;

  /** The latest Global Variables hash for the HEAD of the chain. */
  public latestGlobalVariablesHash: Fr = Fr.ZERO;

  constructor(
    private merkleTreeDb: MerkleTreeDb,
    private l2BlockSource: L2BlockSource,
    private log = createDebugLogger('aztec:world_state'),
  ) {
    const config = getConfigEnvVars();
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

  public async start() {
    if (this.currentState === WorldStateRunningState.STOPPED) {
      throw new Error('Synchroniser already stopped');
    }
    if (this.currentState !== WorldStateRunningState.IDLE) {
      return this.syncPromise;
    }

    // get the current latest block number
    this.latestBlockNumberAtStart = await this.l2BlockSource.getBlockHeight();
    this.latestGlobalVariablesHash = await computeGlobalVariablesHash();

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
        const blocks = await this.l2BlockDownloader.getL2Blocks();
        await this.handleL2Blocks(blocks);
      }
    };
    this.runningPromise = blockProcess();
    this.l2BlockDownloader.start(blockToDownloadFrom);
    this.log(`Started block downloader from block ${blockToDownloadFrom}`);
    return this.syncPromise;
  }

  public async stop() {
    this.log('Stopping world state...');
    this.stopping = true;
    await this.l2BlockDownloader.stop();
    await this.runningPromise;
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
   * Handles a list of L2 blocks (i.e. Inserts the new commitments into the merkle tree).
   * @param l2Blocks - The L2 blocks to handle.
   */
  private async handleL2Blocks(l2Blocks: L2Block[]) {
    for (const l2Block of l2Blocks) {
      await this.handleL2Block(l2Block);
    }
  }

  /**
   * Handles a single L2 block (i.e. Inserts the new commitments into the merkle tree).
   * @param l2Block - The L2 block to handle.
   */
  private async handleL2Block(l2Block: L2Block) {
    await this.merkleTreeDb.handleL2Block(l2Block);
    this.currentL2BlockNum = l2Block.number;
    this.latestGlobalVariablesHash = await computeGlobalVariablesHash(l2Block.globalVariables);
    this.log(`Synced global variables with hash ${this.latestGlobalVariablesHash.toString()}`);
    if (
      this.currentState === WorldStateRunningState.SYNCHING &&
      this.currentL2BlockNum >= this.latestBlockNumberAtStart
    ) {
      this.setCurrentState(WorldStateRunningState.RUNNING);
      if (this.syncResolve !== undefined) {
        this.syncResolve();
      }
    }
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

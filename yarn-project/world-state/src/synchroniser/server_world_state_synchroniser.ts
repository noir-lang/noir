import { WorldStateRunningState, WorldStateStatus, WorldStateSynchroniser } from './world_state_synchroniser.js';
import { MerkleTreeDb, MerkleTreeId } from '@aztec/merkle-tree';
import { L2BlockSource, L2BlockDownloader, L2Block } from '@aztec/archiver';
import { createDebugLogger } from '@aztec/foundation';

/**
 * Synchronises the world state with the L2 blocks from a L2BlockSource.
 * The synchroniser will download the L2 blocks from the L2BlockSource and insert the new commitments into the merkle
 * tree.
 */
export class ServerWorldStateSynchroniser implements WorldStateSynchroniser {
  private currentL2BlockNum = -1;
  private latestBlockNumberAtStart = -1;
  private l2BlockDownloader: L2BlockDownloader;
  private syncPromise: Promise<void> = Promise.resolve();
  private syncResolve?: () => void = undefined;
  private stopping = false;
  private runningPromise: Promise<void> = Promise.resolve();
  private currentState: WorldStateRunningState = WorldStateRunningState.IDLE;

  constructor(
    private merkleTreeDb: MerkleTreeDb,
    private l2BlockSource: L2BlockSource,
    maxQueueSize = 1000,
    pollIntervalMS = 10000,
    private log = createDebugLogger('aztec:world_state'),
  ) {
    this.l2BlockDownloader = new L2BlockDownloader(l2BlockSource, maxQueueSize, pollIntervalMS);
  }

  /**
   * Starts the synchroniser.
   * @param from - The block number to start downloading from. Defaults to 0.
   * @returns A promise that resolves once the initial sync is completed.
   */
  public async start(from = 0) {
    if (this.currentState === WorldStateRunningState.STOPPED) {
      throw new Error('Synchroniser already stopped');
    }
    if (this.currentState !== WorldStateRunningState.IDLE) {
      return this.syncPromise;
    }

    // get the current latest block number
    this.latestBlockNumberAtStart = await this.l2BlockSource.getLatestBlockNum();

    // if there are blocks to be retrieved, go to a synching state
    if (from < this.latestBlockNumberAtStart) {
      this.setCurrentState(WorldStateRunningState.SYNCHING);
      this.syncPromise = new Promise(resolve => {
        this.syncResolve = resolve;
      });
      this.log(`starting sync from ${from}, latest block ${this.latestBlockNumberAtStart}`);
    } else {
      // if no blocks to be retrieved, go straight to running
      this.setCurrentState(WorldStateRunningState.RUNNING);
      this.currentL2BlockNum = this.latestBlockNumberAtStart;
      this.syncPromise = Promise.resolve();
      this.log(`already synched to latest block at ${this.latestBlockNumberAtStart}`);
    }

    // start looking for further blocks
    const blockProcess = async () => {
      while (!this.stopping) {
        const blocks = await this.l2BlockDownloader.getL2Blocks();
        await this.handleL2Blocks(blocks);
      }
    };
    this.runningPromise = blockProcess();
    this.l2BlockDownloader.start(from);
    this.log(`started block downloader from block ${from}`);
    return this.syncPromise;
  }

  /**
   * Stops the synchroniser.
   */
  public async stop() {
    this.log('stopping world state...');
    this.stopping = true;
    await this.l2BlockDownloader.stop();
    await this.runningPromise;
    this.setCurrentState(WorldStateRunningState.STOPPED);
  }

  /**
   * Returns the current status of the synchroniser.
   * @returns The current status of the synchroniser.
   */
  public status(): Promise<WorldStateStatus> {
    const status = {
      syncedToL2Block: this.currentL2BlockNum,
      state: this.currentState,
    } as WorldStateStatus;
    return Promise.resolve(status);
  }

  /**
   * Handles a list of L2 blocks (i.e. Inserts the new commitments into the merkle tree).
   * @param l2blocks - The L2 blocks to handle.
   */
  private async handleL2Blocks(l2blocks: L2Block[]) {
    for (const l2block of l2blocks) {
      await this.handleL2Block(l2block);
    }
  }

  /**
   * Handles a single L2 block (i.e. Inserts the new commitments into the merkle tree).
   * @param l2block - The L2 block to handle.
   */
  private async handleL2Block(l2block: L2Block) {
    this.log(`committing block ${l2block.number}`);
    await this.merkleTreeDb.appendLeaves(MerkleTreeId.CONTRACT_TREE, l2block.newContracts);
    await this.merkleTreeDb.commit();
    this.log(`committed block ${l2block.number} to world state`);
    this.currentL2BlockNum = l2block.number;
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
    this.log(`moved to state ${WorldStateRunningState[this.currentState]}`);
  }
}

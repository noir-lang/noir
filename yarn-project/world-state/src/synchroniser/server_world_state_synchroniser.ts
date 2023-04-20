import { createDebugLogger } from '@aztec/foundation';
import { L2Block, L2BlockDownloader, L2BlockSource } from '@aztec/types';
import { MerkleTreeDb, MerkleTreeId, MerkleTreeOperations } from '../index.js';
import { MerkleTreeOperationsFacade } from '../merkle-tree/merkle_tree_operations_facade.js';
import { WorldStateRunningState, WorldStateStatus, WorldStateSynchroniser } from './world_state_synchroniser.js';
import { getConfigEnvVars } from './config.js';

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

  constructor(
    private merkleTreeDb: MerkleTreeDb,
    private l2BlockSource: L2BlockSource,
    private log = createDebugLogger('aztec:world_state'),
  ) {
    const config = getConfigEnvVars();
    this.l2BlockDownloader = new L2BlockDownloader(l2BlockSource, config.l2QueueSize, config.checkInterval);
  }

  public getLatest(): MerkleTreeOperations {
    return new MerkleTreeOperationsFacade(this.merkleTreeDb, true);
  }

  public getCommitted(): MerkleTreeOperations {
    return new MerkleTreeOperationsFacade(this.merkleTreeDb, false);
  }

  /**
   * Starts the synchroniser.
   * @returns A promise that resolves once the initial sync is completed.
   */
  public async start() {
    if (this.currentState === WorldStateRunningState.STOPPED) {
      throw new Error('Synchroniser already stopped');
    }
    if (this.currentState !== WorldStateRunningState.IDLE) {
      return this.syncPromise;
    }

    // get the current latest block number
    this.latestBlockNumberAtStart = await this.l2BlockSource.getBlockHeight();

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

  /**
   * Stops the synchroniser.
   */
  public async stop() {
    this.log('Stopping world state...');
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
    const compareRoot = async (root: Buffer, treeId: MerkleTreeId) => {
      const treeInfo = await this.merkleTreeDb.getTreeInfo(treeId, true);
      return treeInfo.root.equals(root);
    };
    const rootChecks = await Promise.all([
      compareRoot(l2Block.endContractTreeSnapshot.root.toBuffer(), MerkleTreeId.CONTRACT_TREE),
      compareRoot(l2Block.endNullifierTreeSnapshot.root.toBuffer(), MerkleTreeId.NULLIFIER_TREE),
      compareRoot(l2Block.endPrivateDataTreeSnapshot.root.toBuffer(), MerkleTreeId.PRIVATE_DATA_TREE),
      compareRoot(
        l2Block.endTreeOfHistoricContractTreeRootsSnapshot.root.toBuffer(),
        MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
      ),
      compareRoot(
        l2Block.endTreeOfHistoricPrivateDataTreeRootsSnapshot.root.toBuffer(),
        MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
      ),
    ]);
    const ourBlock = rootChecks.every(x => x);
    if (ourBlock) {
      this.log(`Block ${l2Block.number} is ours, committing world state..`);
      await this.merkleTreeDb.commit();
    } else {
      this.log(`Block ${l2Block.number} is not ours, rolling back world state and committing state from chain..`);
      await this.merkleTreeDb.rollback();

      for (const [tree, leaves] of [
        [MerkleTreeId.CONTRACT_TREE, l2Block.newContracts],
        [MerkleTreeId.NULLIFIER_TREE, l2Block.newNullifiers],
        [MerkleTreeId.PRIVATE_DATA_TREE, l2Block.newCommitments],
      ] as const) {
        await this.merkleTreeDb.appendLeaves(
          tree,
          leaves.map(fr => fr.toBuffer()),
        );
      }

      for (const [newTree, rootTree] of [
        [MerkleTreeId.PRIVATE_DATA_TREE, MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE],
        [MerkleTreeId.CONTRACT_TREE, MerkleTreeId.CONTRACT_TREE_ROOTS_TREE],
      ] as const) {
        const newTreeInfo = await this.merkleTreeDb.getTreeInfo(newTree, true);
        await this.merkleTreeDb.appendLeaves(rootTree, [newTreeInfo.root]);
      }

      await this.merkleTreeDb.commit();
    }
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

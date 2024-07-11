import { type L2Block, L2BlockDownloader, type L2BlockSource, type Tx, type TxHash } from '@aztec/circuit-types';
import { INITIAL_L2_BLOCK_NUM } from '@aztec/circuits.js/constants';
import { createDebugLogger } from '@aztec/foundation/log';
import { type AztecKVStore, type AztecSingleton } from '@aztec/kv-store';

import { getP2PConfigEnvVars } from '../config.js';
import type { P2PService } from '../service/service.js';
import { type TxPool } from '../tx_pool/index.js';

/**
 * Enum defining the possible states of the p2p client.
 */
export enum P2PClientState {
  IDLE,
  SYNCHING,
  RUNNING,
  STOPPED,
}

/**
 * The synchronization status of the P2P client.
 */
export interface P2PSyncState {
  /**
   * The current state of the p2p client.
   */
  state: P2PClientState;
  /**
   * The block number that the p2p client is synced to.
   */
  syncedToL2Block: number;
}

/**
 * Interface of a P2P client.
 **/
export interface P2P {
  /**
   * Verifies the 'tx' and, if valid, adds it to local tx pool and forwards it to other peers.
   * @param tx - The transaction.
   **/
  sendTx(tx: Tx): Promise<void>;

  /**
   * Deletes 'txs' from the pool, given hashes.
   * NOT used if we use sendTx as reconcileTxPool will handle this.
   * @param txHashes - Hashes to check.
   **/
  deleteTxs(txHashes: TxHash[]): Promise<void>;

  /**
   * Returns all transactions in the transaction pool.
   * @returns An array of Txs.
   */
  getTxs(): Promise<Tx[]>;

  /**
   * Returns a transaction in the transaction pool by its hash.
   * @param txHash  - Hash of tx to return.
   * @returns A single tx or undefined.
   */
  getTxByHash(txHash: TxHash): Promise<Tx | undefined>;

  /**
   * Starts the p2p client.
   * @returns A promise signalling the completion of the block sync.
   */
  start(): Promise<void>;

  /**
   * Stops the p2p client.
   * @returns A promise signalling the completion of the stop process.
   */
  stop(): Promise<void>;

  /**
   * Indicates if the p2p client is ready for transaction submission.
   * @returns A boolean flag indicating readiness.
   */
  isReady(): Promise<boolean>;

  /**
   * Returns the current status of the p2p client.
   */
  getStatus(): Promise<P2PSyncState>;
}

/**
 * The P2P client implementation.
 */
export class P2PClient implements P2P {
  /**
   * L2 Block download that p2p client uses to stay in sync with latest blocks.
   */
  private blockDownloader: L2BlockDownloader;

  /**
   * Property that indicates whether the client is running.
   */
  private stopping = false;

  /**
   * The JS promise that will be running to keep the client's data in sync. Can be interrupted if the client is stopped.
   */
  private runningPromise!: Promise<void>;

  private currentState = P2PClientState.IDLE;
  private syncPromise = Promise.resolve();
  private latestBlockNumberAtStart = -1;
  private syncResolve?: () => void = undefined;
  private synchedBlockNumber: AztecSingleton<number>;

  /**
   * In-memory P2P client constructor.
   * @param store - The client's instance of the KV store.
   * @param l2BlockSource - P2P client's source for fetching existing blocks.
   * @param txPool - The client's instance of a transaction pool. Defaults to in-memory implementation.
   * @param p2pService - The concrete instance of p2p networking to use.
   * @param log - A logger.
   */
  constructor(
    store: AztecKVStore,
    private l2BlockSource: L2BlockSource,
    private txPool: TxPool,
    private p2pService: P2PService,
    private log = createDebugLogger('aztec:p2p'),
  ) {
    const { p2pBlockCheckIntervalMS: checkInterval, p2pL2QueueSize } = getP2PConfigEnvVars();
    this.blockDownloader = new L2BlockDownloader(l2BlockSource, {
      maxQueueSize: p2pL2QueueSize,
      pollIntervalMS: checkInterval,
    });
    this.synchedBlockNumber = store.openSingleton('p2p_pool_last_l2_block');
  }

  /**
   * Starts the P2P client.
   * @returns An empty promise signalling the synching process.
   */
  public async start() {
    if (this.currentState === P2PClientState.STOPPED) {
      throw new Error('P2P client already stopped');
    }
    if (this.currentState !== P2PClientState.IDLE) {
      return this.syncPromise;
    }

    // get the current latest block number
    this.latestBlockNumberAtStart = await this.l2BlockSource.getBlockNumber();

    const blockToDownloadFrom = this.getSyncedBlockNum() + 1;

    // if there are blocks to be retrieved, go to a synching state
    if (blockToDownloadFrom <= this.latestBlockNumberAtStart) {
      this.setCurrentState(P2PClientState.SYNCHING);
      this.syncPromise = new Promise(resolve => {
        this.syncResolve = resolve;
      });
      this.log.verbose(`Starting sync from ${blockToDownloadFrom}, latest block ${this.latestBlockNumberAtStart}`);
    } else {
      // if no blocks to be retrieved, go straight to running
      this.setCurrentState(P2PClientState.RUNNING);
      this.syncPromise = Promise.resolve();
      await this.p2pService.start();
      this.log.verbose(
        `Next block ${blockToDownloadFrom} already beyond latest block at ${this.latestBlockNumberAtStart}`,
      );
    }

    // publish any txs in TxPool after its doing initial sync
    this.syncPromise = this.syncPromise.then(() => this.publishStoredTxs());

    // start looking for further blocks
    const blockProcess = async () => {
      while (!this.stopping) {
        const blocks = await this.blockDownloader.getBlocks();
        await this.handleL2Blocks(blocks);
      }
    };
    this.runningPromise = blockProcess();
    this.blockDownloader.start(blockToDownloadFrom);
    this.log.verbose(`Started block downloader from block ${blockToDownloadFrom}`);

    return this.syncPromise;
  }

  /**
   * Allows consumers to stop the instance of the P2P client.
   * 'ready' will now return 'false' and the running promise that keeps the client synced is interrupted.
   */
  public async stop() {
    this.log.debug('Stopping p2p client...');
    this.stopping = true;
    await this.p2pService.stop();
    this.log.debug('Stopped p2p service');
    await this.blockDownloader.stop();
    this.log.debug('Stopped block downloader');
    await this.runningPromise;
    this.setCurrentState(P2PClientState.STOPPED);
    this.log.info('P2P client stopped.');
  }

  /**
   * Returns all transactions in the transaction pool.
   * @returns An array of Txs.
   */
  public getTxs(): Promise<Tx[]> {
    return Promise.resolve(this.txPool.getAllTxs());
  }

  /**
   * Returns a transaction in the transaction pool by its hash.
   * @param txHash - Hash of the transaction to look for in the pool.
   * @returns A single tx or undefined.
   */
  getTxByHash(txHash: TxHash): Promise<Tx | undefined> {
    return Promise.resolve(this.txPool.getTxByHash(txHash));
  }

  /**
   * Verifies the 'tx' and, if valid, adds it to local tx pool and forwards it to other peers.
   * @param tx - The tx to verify.
   * @returns Empty promise.
   **/
  public async sendTx(tx: Tx): Promise<void> {
    const ready = await this.isReady();
    if (!ready) {
      throw new Error('P2P client not ready');
    }
    await this.txPool.addTxs([tx]);
    this.p2pService.propagateTx(tx);
  }

  /**
   * Deletes the 'txs' from the pool.
   * NOT used if we use sendTx as reconcileTxPool will handle this.
   * @param txHashes - Hashes of the transactions to delete.
   * @returns Empty promise.
   **/
  public async deleteTxs(txHashes: TxHash[]): Promise<void> {
    const ready = await this.isReady();
    if (!ready) {
      throw new Error('P2P client not ready');
    }
    await this.txPool.deleteTxs(txHashes);
  }

  /**
   * Public function to check if the p2p client is fully synced and ready to receive txs.
   * @returns True if the P2P client is ready to receive txs.
   */
  public isReady() {
    return Promise.resolve(this.currentState === P2PClientState.RUNNING);
  }

  /**
   * Public function to check the latest block number that the P2P client is synced to.
   * @returns Block number of latest L2 Block we've synced with.
   */
  public getSyncedBlockNum() {
    return this.synchedBlockNumber.get() ?? INITIAL_L2_BLOCK_NUM - 1;
  }

  /**
   * Method to check the status the p2p client.
   * @returns Information about p2p client status: state & syncedToBlockNum.
   */
  public getStatus(): Promise<P2PSyncState> {
    return Promise.resolve({
      state: this.currentState,
      syncedToL2Block: this.getSyncedBlockNum(),
    } as P2PSyncState);
  }

  /**
   * Internal method that uses the provided blocks to check against the client's tx pool.
   * @param blocks - A list of existing blocks with txs that the P2P client needs to ensure the tx pool is reconciled with.
   * @returns Empty promise.
   */
  private async reconcileTxPool(blocks: L2Block[]): Promise<void> {
    for (const block of blocks) {
      const txHashes = block.body.txEffects.map(txEffect => txEffect.txHash);
      await this.txPool.deleteTxs(txHashes);
    }
  }

  /**
   * Method for processing new blocks.
   * @param blocks - A list of existing blocks with txs that the P2P client needs to ensure the tx pool is reconciled with.
   * @returns Empty promise.
   */
  private async handleL2Blocks(blocks: L2Block[]): Promise<void> {
    if (!blocks.length) {
      return Promise.resolve();
    }
    await this.reconcileTxPool(blocks);
    const lastBlockNum = blocks[blocks.length - 1].number;
    await this.synchedBlockNumber.set(lastBlockNum);
    this.log.debug(`Synched to block ${lastBlockNum}`);

    if (this.currentState === P2PClientState.SYNCHING && lastBlockNum >= this.latestBlockNumberAtStart) {
      this.setCurrentState(P2PClientState.RUNNING);
      if (this.syncResolve !== undefined) {
        this.syncResolve();
        await this.p2pService.start();
      }
    }
  }

  /**
   * Method to set the value of the current state.
   * @param newState - New state value.
   */
  private setCurrentState(newState: P2PClientState) {
    this.currentState = newState;
    this.log.debug(`Moved to state ${P2PClientState[this.currentState]}`);
  }

  private async publishStoredTxs() {
    if (!this.isReady()) {
      return;
    }

    const txs = this.txPool.getAllTxs();
    if (txs.length > 0) {
      this.log.debug(`Publishing ${txs.length} previously stored txs`);
      await Promise.all(txs.map(tx => this.p2pService.propagateTx(tx)));
    }
  }
}

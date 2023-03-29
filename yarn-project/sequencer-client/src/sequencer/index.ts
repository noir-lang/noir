import { Tx } from '@aztec/tx';
import { P2P } from '@aztec/p2p';
import { WorldStateSynchroniser, WorldStateStatus } from '@aztec/world-state';
import { RunningPromise } from '../deps/running_promise.js';
import { L1Publisher } from '../publisher/l1-publisher.js';
import { createDebugLogger } from '@aztec/foundation';
import { BlockBuilder } from './block_builder.js';
import { SequencerConfig } from './config.js';

/**
 * Sequencer client
 * - Wins a period of time to become the sequencer (depending on finalised protocol).
 * - Chooses a set of txs from the tx pool to be in the rollup.
 * - Simulate the rollup of txs.
 * - Adds proof requests to the request pool (not for this milestone).
 * - Receives results to those proofs from the network (repeats as necessary) (not for this milestone).
 * - Publishes L1 tx(s) to the rollup contract via RollupPublisher.
 * - For this milestone, the sequencer will just simulate and publish a 1x1 rollup and publish it to L1.
 */
export class Sequencer {
  private runningPromise?: RunningPromise;
  private pollingIntervalMs: number;
  private lastBlockNumber = -1;
  private state = SequencerState.STOPPED;

  constructor(
    private publisher: L1Publisher,
    private p2pClient: P2P,
    private worldState: WorldStateSynchroniser,
    config?: SequencerConfig,
    private log = createDebugLogger('aztec:sequencer'),
  ) {
    this.pollingIntervalMs = config?.transactionPollingInterval ?? 1_000;
  }

  public async start() {
    // TODO: Should we wait for worldstate to be ready, or is the caller expected to run await start?
    this.lastBlockNumber = await this.worldState.status().then((s: WorldStateStatus) => s.syncedToL2Block);

    this.runningPromise = new RunningPromise(this.work.bind(this), { pollingInterval: this.pollingIntervalMs });
    this.runningPromise.start();
    this.state = SequencerState.IDLE;
  }

  public async stop(): Promise<void> {
    await this.runningPromise?.stop();
    this.publisher.interrupt();
    this.state = SequencerState.STOPPED;
    this.log('Stopped sequencer');
  }

  public status() {
    return { state: this.state };
  }

  /**
   * Grabs a single tx from the p2p client, constructs a block, and pushes it to L1.
   */
  protected async work() {
    try {
      // Update state when the previous block has been synched
      const prevBlockSynched = await this.isBlockSynched();
      if (prevBlockSynched && this.state === SequencerState.PUBLISHING_BLOCK) {
        this.log(`Block has been synched`);
        this.state = SequencerState.IDLE;
      }

      // Do not go forward with new block if the previous one has not been mined and processed
      if (!prevBlockSynched) {
        return;
      }

      this.state = SequencerState.WAITING_FOR_TXS;

      // Get a single tx (for now) to build the new block
      // P2P client is responsible for ensuring this tx is eligible (proof ok, not mined yet, etc)
      const [tx] = await this.p2pClient.getTxs();
      if (!tx) {
        this.log(`No txs in the mempool for a new block`);
        return;
      } else {
        this.log(`Processing tx ${tx.txId.toString('hex')}`);
      }

      this.state = SequencerState.CREATING_BLOCK;

      // Build the new block by running the rollup circuits
      const block = await this.buildBlock(tx);
      this.log(`Assembled block ${block.number}`);

      // Publishes new block to the network and awaits the tx to be mined
      this.state = SequencerState.PUBLISHING_BLOCK;
      const publishedL2Block = await this.publisher.processL2Block(block);
      if (publishedL2Block) {
        this.log(`Successfully published block ${block.number}`);
        this.lastBlockNumber++;
      } else {
        this.log(`Failed to publish block`);
      }

      // Publishes new unverified data to the network and awaits the tx to be mined
      this.state = SequencerState.PUBLISHING_UNVERIFIED_DATA;
      const publishedUnverifiedData = await this.publisher.processUnverifiedData(block.number, tx.unverifiedData);
      if (publishedUnverifiedData) {
        this.log(`Successfully published unverifiedData for block ${block.number}`);
      } else {
        this.log(`Failed to publish unverifiedData for block ${block.number}`);
      }
    } catch (err) {
      this.log(`Error doing work: ${err}`, 'error');
    }
  }

  /**
   * Returns whether the previous block sent has been mined, and all dependencies have caught up with it.
   * @returns Boolean indicating if our dependencies are synched to the latest block.
   */
  protected async isBlockSynched() {
    return (
      (await this.worldState.status().then((s: WorldStateStatus) => s.syncedToL2Block)) >= this.lastBlockNumber &&
      (await this.p2pClient.getStatus().then(s => s.syncedToL2Block)) >= this.lastBlockNumber
    );
  }

  protected async buildBlock(tx: Tx) {
    const blockBuilder = new BlockBuilder(this.worldState, this.lastBlockNumber + 1, tx);
    return await blockBuilder.buildL2Block();
  }
}

export enum SequencerState {
  IDLE,
  WAITING_FOR_TXS,
  CREATING_BLOCK,
  PUBLISHING_BLOCK,
  PUBLISHING_UNVERIFIED_DATA,
  STOPPED,
}

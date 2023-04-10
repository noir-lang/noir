import { createDebugLogger } from '@aztec/foundation';
import { P2P } from '@aztec/p2p';
import { Tx } from '@aztec/tx';
import { MerkleTreeId, WorldStateStatus, WorldStateSynchroniser } from '@aztec/world-state';
import { CircuitBlockBuilder } from '../block_builder/circuit_block_builder.js';
import { RunningPromise } from '../deps/running_promise.js';
import { L1Publisher } from '../publisher/l1-publisher.js';
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
    private blockBuilder: CircuitBlockBuilder,
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
   * Returns true if one of the transaction nullifiers exist.
   * Nullifiers prevent double spends in a private context.
   * @param tx - The transaction.
   * @returns Whether this is a problematic double spend that the L1 contract would reject.
   */
  private async isTxDoubleSpend(tx: Tx): Promise<boolean> {
    // eslint-disable-next-line @typescript-eslint/await-thenable
    for (const nullifier of tx.data.end.newNullifiers) {
      // Skip nullifier if it's empty
      if (nullifier.isZero()) continue;
      // TODO(AD): this is an exhaustive search currently
      if (
        (await this.worldState.getLatest().findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer())) !==
        undefined
      ) {
        // Our nullifier tree has this nullifier already - this transaction is a double spend / not well-formed
        return true;
      }
    }
    return false;
  }

  /**
   * Grabs a single tx from the p2p client, constructs a block, and pushes it to L1.
   */
  protected async work() {
    try {
      // Update state when the previous block has been synced
      const prevBlockSynced = await this.isBlockSynced();
      if (prevBlockSynced && this.state === SequencerState.PUBLISHING_BLOCK) {
        this.log(`Block has been synced`);
        this.state = SequencerState.IDLE;
      }

      // Do not go forward with new block if the previous one has not been mined and processed
      if (!prevBlockSynced) {
        return;
      }

      this.state = SequencerState.WAITING_FOR_TXS;

      // Get a single tx (for now) to build the new block
      // P2P client is responsible for ensuring this tx is eligible (proof ok, not mined yet, etc)
      const [tx] = await this.p2pClient.getTxs();
      const txHash = await tx?.getTxHash();

      if (!tx) {
        return;
      } else {
        this.log(`Processing tx ${txHash}`);
      }
      // TODO(AD) - eventually we should add a limit to how many transactions we
      // skip in this manner and do something more DDOS-proof (like letting the transaction fail and pay a fee).
      if (await this.isTxDoubleSpend(tx)) {
        // Make sure we remove this from the tx pool so we do not consider it again
        this.log(`Deleting double spend tx ${txHash}`);
        await this.p2pClient.deleteTxs([txHash]);
        return;
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
   * @returns Boolean indicating if our dependencies are synced to the latest block.
   */
  protected async isBlockSynced() {
    return (
      (await this.worldState.status().then((s: WorldStateStatus) => s.syncedToL2Block)) >= this.lastBlockNumber &&
      (await this.p2pClient.getStatus().then(s => s.syncedToL2Block)) >= this.lastBlockNumber
    );
  }

  protected async buildBlock(tx: Tx) {
    const [block] = await this.blockBuilder.buildL2Block(this.lastBlockNumber + 1, tx);
    return block;
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

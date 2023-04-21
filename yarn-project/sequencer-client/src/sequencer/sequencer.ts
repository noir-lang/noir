import { RunningPromise, createDebugLogger } from '@aztec/foundation';
import { P2P } from '@aztec/p2p';
import { PrivateTx, PublicTx, Tx, UnverifiedData, isPrivateTx } from '@aztec/types';
import { MerkleTreeId, WorldStateStatus, WorldStateSynchroniser } from '@aztec/world-state';
import times from 'lodash.times';
import { BlockBuilder } from '../block_builder/index.js';
import { L1Publisher } from '../publisher/l1-publisher.js';
import { ceilPowerOfTwo } from '../utils.js';
import { SequencerConfig } from './config.js';
import { ProcessedTx, makeEmptyProcessedTx } from './processed_tx.js';
import { PublicProcessor } from './public_processor.js';

/**
 * Sequencer client
 * - Wins a period of time to become the sequencer (depending on finalised protocol).
 * - Chooses a set of txs from the tx pool to be in the rollup.
 * - Simulate the rollup of txs.
 * - Adds proof requests to the request pool (not for this milestone).
 * - Receives results to those proofs from the network (repeats as necessary) (not for this milestone).
 * - Publishes L1 tx(s) to the rollup contract via RollupPublisher.
 */
export class Sequencer {
  private runningPromise?: RunningPromise;
  private pollingIntervalMs: number;
  private maxTxsPerBlock = 32;
  private lastBlockNumber = -1;
  private state = SequencerState.STOPPED;

  constructor(
    private publisher: L1Publisher,
    private p2pClient: P2P,
    private worldState: WorldStateSynchroniser,
    private blockBuilder: BlockBuilder,
    private publicProcessor: PublicProcessor,
    config?: SequencerConfig,
    private log = createDebugLogger('aztec:sequencer'),
  ) {
    this.pollingIntervalMs = config?.transactionPollingInterval ?? 1_000;
    if (config?.maxTxsPerBlock) {
      this.maxTxsPerBlock = config.maxTxsPerBlock;
    }
  }

  public async start() {
    await this.initialSync();

    this.runningPromise = new RunningPromise(this.work.bind(this), this.pollingIntervalMs);
    this.runningPromise.start();
    this.state = SequencerState.IDLE;
    this.log('Sequencer started');
  }

  public async stop(): Promise<void> {
    await this.runningPromise?.stop();
    this.publisher.interrupt();
    this.state = SequencerState.STOPPED;
    this.log('Stopped sequencer');
  }

  public restart() {
    this.log('Restarting sequencer');
    this.runningPromise!.start();
    this.state = SequencerState.IDLE;
  }

  public status() {
    return { state: this.state };
  }

  protected async initialSync() {
    // TODO: Should we wait for worldstate to be ready, or is the caller expected to run await start?
    this.lastBlockNumber = await this.worldState.status().then((s: WorldStateStatus) => s.syncedToL2Block);
  }

  /**
   * Grabs up to maxTxsPerBlock from the p2p client, constructs a block, and pushes it to L1.
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
      if (!prevBlockSynced) return;

      this.state = SequencerState.WAITING_FOR_TXS;

      // Get txs to build the new block
      const pendingTxs = await this.p2pClient.getTxs();
      if (pendingTxs.length === 0) return;
      this.log(`Processing ${pendingTxs.length} txs from P2P pool`);

      // Filter out invalid txs
      const validTxs = await this.takeValidTxs(pendingTxs);
      if (validTxs.length === 0) {
        this.log(`No valid txs left after processing`);
        return;
      }

      this.log(`Processing txs ${(await Tx.getHashes(validTxs)).join(', ')}`);
      this.state = SequencerState.CREATING_BLOCK;

      // Process public txs and drop the ones that fail processing
      const [processedTxs, failedTxs] = await this.publicProcessor.process(validTxs);
      if (failedTxs.length > 0) {
        this.log(`Dropping failed txs ${(await Tx.getHashes(failedTxs)).join(', ')}`);
        await this.p2pClient.deleteTxs(await Tx.getHashes(failedTxs));
      }

      // Build the new block by running the rollup circuits
      this.log(`Assembling block with txs ${processedTxs.map(tx => tx.hash).join(', ')}`);
      const block = await this.buildBlock(processedTxs);
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

      // Publishes new unverified data for private txs to the network and awaits the tx to be mined
      this.state = SequencerState.PUBLISHING_UNVERIFIED_DATA;
      const unverifiedData = UnverifiedData.join(validTxs.filter(isPrivateTx).map(tx => tx.unverifiedData));
      const publishedUnverifiedData = await this.publisher.processUnverifiedData(block.number, unverifiedData);
      if (publishedUnverifiedData) {
        this.log(`Successfully published unverifiedData for block ${block.number}`);
      } else {
        this.log(`Failed to publish unverifiedData for block ${block.number}`);
      }
    } catch (err) {
      this.log(err, 'error');
      // TODO: Rollback changes to DB
    }
  }

  // TODO: It should be responsibility of the P2P layer to validate txs before passing them on here
  protected async takeValidTxs(txs: Tx[]) {
    const validTxs = [];
    const doubleSpendTxs = [];
    const invalidSigTxs = [];

    // Process txs until we get to maxTxsPerBlock, rejecting double spends in the process
    for (const tx of txs) {
      // TODO(AD) - eventually we should add a limit to how many transactions we
      // skip in this manner and do something more DDOS-proof (like letting the transaction fail and pay a fee).
      if (tx.isPrivate() && (await this.isTxDoubleSpend(tx))) {
        this.log(`Deleting double spend tx ${await tx.getTxHash()}`);
        doubleSpendTxs.push(tx);
        continue;
      }

      if (tx.isPublic() && !(await this.isValidSignature(tx))) {
        this.log(`Deleting invalid signature tx ${await tx.getTxHash()}`);
        invalidSigTxs.push(tx);
        continue;
      }

      validTxs.push(tx);
      if (validTxs.length >= this.maxTxsPerBlock) {
        break;
      }
    }

    // Make sure we remove these from the tx pool so we do not consider it again
    if (doubleSpendTxs.length > 0 || invalidSigTxs.length > 0) {
      await this.p2pClient.deleteTxs(await Tx.getHashes([...doubleSpendTxs, ...invalidSigTxs]));
    }

    return validTxs;
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

  protected async buildBlock(txs: ProcessedTx[]) {
    // Pad the txs array with empty txs to be a power of two, at least 4
    const txsTargetSize = Math.max(ceilPowerOfTwo(txs.length), 4);
    const allTxs = [...txs, ...(await Promise.all(times(txsTargetSize - txs.length, makeEmptyProcessedTx)))];
    const [block] = await this.blockBuilder.buildL2Block(this.lastBlockNumber + 1, allTxs);
    return block;
  }

  /**
   * Returns true if one of the transaction nullifiers exist.
   * Nullifiers prevent double spends in a private context.
   * @param tx - The transaction.
   * @returns Whether this is a problematic double spend that the L1 contract would reject.
   */
  protected async isTxDoubleSpend(tx: PrivateTx): Promise<boolean> {
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

  protected isValidSignature(_tx: PublicTx): Promise<boolean> {
    // TODO: Validate tx ECDSA signature!
    return Promise.resolve(true);
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

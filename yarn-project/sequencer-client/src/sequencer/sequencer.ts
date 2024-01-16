import { L1ToL2MessageSource, L2Block, L2BlockSource, MerkleTreeId, Tx } from '@aztec/circuit-types';
import { L2BlockBuiltStats } from '@aztec/circuit-types/stats';
import { GlobalVariables } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { Timer, elapsed } from '@aztec/foundation/timer';
import { P2P } from '@aztec/p2p';
import { WorldStateStatus, WorldStateSynchronizer } from '@aztec/world-state';

import times from 'lodash.times';

import { BlockBuilder } from '../block_builder/index.js';
import { GlobalVariableBuilder } from '../global_variable_builder/global_builder.js';
import { L1Publisher } from '../publisher/l1-publisher.js';
import { ceilPowerOfTwo } from '../utils.js';
import { SequencerConfig } from './config.js';
import { ProcessedTx } from './processed_tx.js';
import { PublicProcessorFactory } from './public_processor.js';

/**
 * Sequencer client
 * - Wins a period of time to become the sequencer (depending on finalized protocol).
 * - Chooses a set of txs from the tx pool to be in the rollup.
 * - Simulate the rollup of txs.
 * - Adds proof requests to the request pool (not for this milestone).
 * - Receives results to those proofs from the network (repeats as necessary) (not for this milestone).
 * - Publishes L1 tx(s) to the rollup contract via RollupPublisher.
 */
export class Sequencer {
  private runningPromise?: RunningPromise;
  private pollingIntervalMs: number = 1000;
  private maxTxsPerBlock = 32;
  private minTxsPerBLock = 1;
  private lastPublishedBlock = 0;
  private state = SequencerState.STOPPED;

  constructor(
    private publisher: L1Publisher,
    private globalsBuilder: GlobalVariableBuilder,
    private p2pClient: P2P,
    private worldState: WorldStateSynchronizer,
    private blockBuilder: BlockBuilder,
    private l2BlockSource: L2BlockSource,
    private l1ToL2MessageSource: L1ToL2MessageSource,
    private publicProcessorFactory: PublicProcessorFactory,
    config: SequencerConfig = {},
    private log = createDebugLogger('aztec:sequencer'),
  ) {
    this.updateConfig(config);
    this.log(`Initialized sequencer with ${this.minTxsPerBLock}-${this.maxTxsPerBlock} txs per block.`);
  }

  /**
   * Updates sequencer config.
   * @param config - New parameters.
   */
  public updateConfig(config: SequencerConfig) {
    if (config.transactionPollingIntervalMS) {
      this.pollingIntervalMs = config.transactionPollingIntervalMS;
    }
    if (config.maxTxsPerBlock) {
      this.maxTxsPerBlock = config.maxTxsPerBlock;
    }
    if (config.minTxsPerBlock) {
      this.minTxsPerBLock = config.minTxsPerBlock;
    }
  }

  /**
   * Starts the sequencer and moves to IDLE state. Blocks until the initial sync is complete.
   */
  public async start() {
    await this.initialSync();

    this.runningPromise = new RunningPromise(this.work.bind(this), this.pollingIntervalMs);
    this.runningPromise.start();
    this.state = SequencerState.IDLE;
    this.log('Sequencer started');
  }

  /**
   * Stops the sequencer from processing txs and moves to STOPPED state.
   */
  public async stop(): Promise<void> {
    this.log(`Stopping sequencer`);
    await this.runningPromise?.stop();
    this.publisher.interrupt();
    this.state = SequencerState.STOPPED;
    this.log('Stopped sequencer');
  }

  /**
   * Starts a previously stopped sequencer.
   */
  public restart() {
    this.log('Restarting sequencer');
    this.publisher.restart();
    this.runningPromise!.start();
    this.state = SequencerState.IDLE;
  }

  /**
   * Returns the current state of the sequencer.
   * @returns An object with a state entry with one of SequencerState.
   */
  public status() {
    return { state: this.state };
  }

  protected async initialSync() {
    // TODO: Should we wait for world state to be ready, or is the caller expected to run await start?
    this.lastPublishedBlock = await this.worldState.status().then((s: WorldStateStatus) => s.syncedToL2Block);
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
      if (!prevBlockSynced) {
        return;
      }

      const workTimer = new Timer();
      this.state = SequencerState.WAITING_FOR_TXS;

      // Get txs to build the new block
      const pendingTxs = await this.p2pClient.getTxs();
      if (pendingTxs.length < this.minTxsPerBLock) {
        return;
      }
      this.log.info(`Retrieved ${pendingTxs.length} txs from P2P pool`);

      const blockNumber = (await this.l2BlockSource.getBlockNumber()) + 1;

      /**
       * We'll call this function before running expensive operations to avoid wasted work.
       */
      const assertBlockHeight = async () => {
        const currentBlockNumber = await this.l2BlockSource.getBlockNumber();
        if (currentBlockNumber + 1 !== blockNumber) {
          throw new Error('New block was emitted while building block');
        }
      };

      const newGlobalVariables = await this.globalsBuilder.buildGlobalVariables(new Fr(blockNumber));

      // Filter out invalid txs
      // TODO: It should be responsibility of the P2P layer to validate txs before passing them on here
      const validTxs = await this.takeValidTxs(pendingTxs, newGlobalVariables);
      if (validTxs.length < this.minTxsPerBLock) {
        return;
      }

      this.log.info(`Building block ${blockNumber} with ${validTxs.length} transactions`);
      this.state = SequencerState.CREATING_BLOCK;

      const prevGlobalVariables =
        (await this.l2BlockSource.getBlock(-1))?.header.globalVariables ?? GlobalVariables.empty();

      // Process txs and drop the ones that fail processing
      // We create a fresh processor each time to reset any cached state (eg storage writes)
      const processor = await this.publicProcessorFactory.create(prevGlobalVariables, newGlobalVariables);
      const [publicProcessorDuration, [processedTxs, failedTxs]] = await elapsed(() => processor.process(validTxs));
      if (failedTxs.length > 0) {
        const failedTxData = failedTxs.map(fail => fail.tx);
        this.log(`Dropping failed txs ${(await Tx.getHashes(failedTxData)).join(', ')}`);
        await this.p2pClient.deleteTxs(await Tx.getHashes(failedTxData));
      }

      // Only accept processed transactions that are not double-spends,
      // public functions emitting nullifiers would pass earlier check but fail here.
      // Note that we're checking all nullifiers generated in the private execution twice,
      // we could store the ones already checked and skip them here as an optimization.
      const processedValidTxs = await this.takeValidTxs(processedTxs, newGlobalVariables);

      if (processedValidTxs.length === 0) {
        this.log('No txs processed correctly to build block. Exiting');
        return;
      }

      await assertBlockHeight();

      // Get l1 to l2 messages from the contract
      this.log('Requesting L1 to L2 messages from contract');
      const l1ToL2Messages = await this.getPendingL1ToL2Messages();
      this.log('Successfully retrieved L1 to L2 messages from contract');

      // Build the new block by running the rollup circuits
      this.log(`Assembling block with txs ${processedValidTxs.map(tx => tx.hash).join(', ')}`);

      await assertBlockHeight();

      const emptyTx = await processor.makeEmptyProcessedTx();
      const [rollupCircuitsDuration, block] = await elapsed(() =>
        this.buildBlock(processedValidTxs, l1ToL2Messages, emptyTx, newGlobalVariables),
      );

      this.log(`Assembled block ${block.number}`, {
        eventName: 'l2-block-built',
        duration: workTimer.ms(),
        publicProcessDuration: publicProcessorDuration,
        rollupCircuitsDuration: rollupCircuitsDuration,
        ...block.getStats(),
      } satisfies L2BlockBuiltStats);

      await assertBlockHeight();

      await this.publishExtendedContractData(processedValidTxs, block);

      await assertBlockHeight();

      await this.publishL2Block(block);
      this.log.info(`Submitted rollup block ${block.number} with ${processedValidTxs.length} transactions`);
    } catch (err) {
      this.log.error(`Rolling back world state DB due to error assembling block`, (err as any).stack);
      await this.worldState.getLatest().rollback();
    }
  }

  /**
   * Gets new extended contract data from the txs and publishes it on chain.
   * @param validTxs - The set of real transactions being published as part of the block.
   * @param block - The L2Block to be published.
   */
  protected async publishExtendedContractData(validTxs: ProcessedTx[], block: L2Block) {
    // Publishes contract data for txs to the network and awaits the tx to be mined
    this.state = SequencerState.PUBLISHING_CONTRACT_DATA;
    const newContracts = validTxs.flatMap(tx => tx.newContracts).filter(cd => !cd.isEmpty());

    if (newContracts.length === 0) {
      this.log.debug(`No new contracts to publish in block ${block.number}`);
      return;
    }

    const blockCalldataHash = block.getCalldataHash();
    this.log.info(`Publishing ${newContracts.length} contracts in block ${block.number}`);

    const publishedContractData = await this.publisher.processNewContractData(
      block.number,
      blockCalldataHash,
      newContracts,
    );

    if (publishedContractData) {
      this.log(`Successfully published new contract data for block ${block.number}`);
    } else if (!publishedContractData && newContracts.length) {
      this.log(`Failed to publish new contract data for block ${block.number}`);
    }
  }

  /**
   * Publishes the L2Block to the rollup contract.
   * @param block - The L2Block to be published.
   */
  protected async publishL2Block(block: L2Block) {
    // Publishes new block to the network and awaits the tx to be mined
    this.state = SequencerState.PUBLISHING_BLOCK;
    const publishedL2Block = await this.publisher.processL2Block(block);
    if (publishedL2Block) {
      this.log(`Successfully published block ${block.number}`);
      this.lastPublishedBlock = block.number;
    } else {
      throw new Error(`Failed to publish block`);
    }
  }

  protected async takeValidTxs<T extends Tx | ProcessedTx>(txs: T[], globalVariables: GlobalVariables): Promise<T[]> {
    const validTxs: T[] = [];
    const txsToDelete = [];
    const thisBlockNullifiers: Set<bigint> = new Set();

    // Process txs until we get to maxTxsPerBlock, rejecting double spends in the process
    for (const tx of txs) {
      if (tx.data.constants.txContext.chainId.value !== globalVariables.chainId.value) {
        this.log(
          `Deleting tx for incorrect chain ${tx.data.constants.txContext.chainId.toString()}, tx hash ${await Tx.getHash(
            tx,
          )}`,
        );
        txsToDelete.push(tx);
        continue;
      }
      if (await this.isTxDoubleSpend(tx)) {
        this.log(`Deleting double spend tx ${await Tx.getHash(tx)}`);
        txsToDelete.push(tx);
        continue;
      } else if (this.isTxDoubleSpendSameBlock(tx, thisBlockNullifiers)) {
        // We don't drop these txs from the p2p pool immediately since they become valid
        // again if the current block fails to be published for some reason.
        this.log(`Skipping tx with double-spend for this same block ${await Tx.getHash(tx)}`);
        continue;
      }

      tx.data.end.newNullifiers.forEach(n => thisBlockNullifiers.add(n.value.toBigInt()));
      validTxs.push(tx);
      if (validTxs.length >= this.maxTxsPerBlock) {
        break;
      }
    }

    // Make sure we remove these from the tx pool so we do not consider it again
    if (txsToDelete.length > 0) {
      await this.p2pClient.deleteTxs(await Tx.getHashes([...txsToDelete]));
    }

    return validTxs;
  }

  /**
   * Returns whether the previous block sent has been mined, and all dependencies have caught up with it.
   * @returns Boolean indicating if our dependencies are synced to the latest block.
   */
  protected async isBlockSynced() {
    const syncedBlocks = await Promise.all([
      this.worldState.status().then((s: WorldStateStatus) => s.syncedToL2Block),
      this.p2pClient.getStatus().then(s => s.syncedToL2Block),
      this.l2BlockSource.getBlockNumber(),
      this.l1ToL2MessageSource.getBlockNumber(),
    ]);
    const min = Math.min(...syncedBlocks);
    return min >= this.lastPublishedBlock;
  }

  /**
   * Pads the set of txs to a power of two and assembles a block by calling the block builder.
   * @param txs - Processed txs to include in the next block.
   * @param newL1ToL2Messages - L1 to L2 messages to be part of the block.
   * @param emptyTx - Empty tx to repeat at the end of the block to pad to a power of two.
   * @param globalVariables - Global variables to use in the block.
   * @returns The new block.
   */
  protected async buildBlock(
    txs: ProcessedTx[],
    newL1ToL2Messages: Fr[],
    emptyTx: ProcessedTx,
    globalVariables: GlobalVariables,
  ) {
    // Pad the txs array with empty txs to be a power of two, at least 2
    const txsTargetSize = Math.max(ceilPowerOfTwo(txs.length), 2);
    const emptyTxCount = txsTargetSize - txs.length;

    const allTxs = [...txs, ...times(emptyTxCount, () => emptyTx)];
    this.log(`Building block ${globalVariables.blockNumber}`);

    const [block] = await this.blockBuilder.buildL2Block(globalVariables, allTxs, newL1ToL2Messages);
    return block;
  }

  /**
   * Calls the archiver to pull upto `NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP` message keys
   * (archiver returns the top messages sorted by fees)
   * @returns An array of L1 to L2 messages' messageKeys
   */
  protected async getPendingL1ToL2Messages(): Promise<Fr[]> {
    return await this.l1ToL2MessageSource.getPendingL1ToL2Messages();
  }

  /**
   * Returns true if one of the tx nullifiers exist on the block being built.
   * @param tx - The tx to test.
   * @param thisBlockNullifiers - The nullifiers added so far.
   */
  protected isTxDoubleSpendSameBlock(tx: Tx | ProcessedTx, thisBlockNullifiers: Set<bigint>): boolean {
    // We only consider non-empty nullifiers
    const newNullifiers = tx.data.end.newNullifiers.filter(n => !n.isEmpty());

    for (const nullifier of newNullifiers) {
      if (thisBlockNullifiers.has(nullifier.value.toBigInt())) {
        return true;
      }
    }
    return false;
  }

  /**
   * Returns true if one of the transaction nullifiers exist.
   * Nullifiers prevent double spends in a private context.
   * @param tx - The transaction.
   * @returns Whether this is a problematic double spend that the L1 contract would reject.
   */
  protected async isTxDoubleSpend(tx: Tx | ProcessedTx): Promise<boolean> {
    // We only consider non-empty nullifiers
    const newNullifiers = tx.data.end.newNullifiers.filter(n => !n.isEmpty());

    // Ditch this tx if it has a repeated nullifiers
    const uniqNullifiers = new Set(newNullifiers.map(n => n.value.toBigInt()));
    if (uniqNullifiers.size !== newNullifiers.length) {
      return true;
    }

    for (const nullifier of newNullifiers) {
      // TODO(AD): this is an exhaustive search currently
      const db = this.worldState.getLatest();
      const indexInDb = await db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
      if (indexInDb !== undefined) {
        return true;
      }
    }
    return false;
  }
}

/**
 * State of the sequencer.
 */
export enum SequencerState {
  /**
   * Will move to WAITING_FOR_TXS after a configured amount of time.
   */
  IDLE,
  /**
   * Polling the P2P module for txs to include in a block. Will move to CREATING_BLOCK if there are valid txs to include, or back to IDLE otherwise.
   */
  WAITING_FOR_TXS,
  /**
   * Creating a new L2 block. Includes processing public function calls and running rollup circuits. Will move to PUBLISHING_CONTRACT_DATA.
   */
  CREATING_BLOCK,
  /**
   * Sending the tx to L1 with encrypted logs and awaiting it to be mined. Will move back to PUBLISHING_BLOCK once finished.
   */
  PUBLISHING_CONTRACT_DATA,
  /**
   * Sending the tx to L1 with the L2 block data and awaiting it to be mined. Will move to IDLE.
   */
  PUBLISHING_BLOCK,
  /**
   * Sequencer is stopped and not processing any txs from the pool.
   */
  STOPPED,
}

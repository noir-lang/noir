import {
  type L1ToL2MessageSource,
  type L2Block,
  type L2BlockSource,
  type ProcessedTx,
  Tx,
  type TxValidator,
} from '@aztec/circuit-types';
import {
  type AllowedElement,
  BlockProofError,
  type BlockProver,
  PROVING_STATUS,
} from '@aztec/circuit-types/interfaces';
import { type L2BlockBuiltStats } from '@aztec/circuit-types/stats';
import { AztecAddress, EthAddress, type GlobalVariables, type Header } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { Timer, elapsed } from '@aztec/foundation/timer';
import { type P2P } from '@aztec/p2p';
import { type PublicProcessorFactory } from '@aztec/simulator';
import { Attributes, type TelemetryClient, type Tracer, trackSpan } from '@aztec/telemetry-client';
import { type WorldStateStatus, type WorldStateSynchronizer } from '@aztec/world-state';

import { type GlobalVariableBuilder } from '../global_variable_builder/global_builder.js';
import { type L1Publisher } from '../publisher/l1-publisher.js';
import { type TxValidatorFactory } from '../tx_validator/tx_validator_factory.js';
import { type SequencerConfig } from './config.js';

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
  private minSecondsBetweenBlocks = 0;
  private maxSecondsBetweenBlocks = 0;
  // TODO: zero values should not be allowed for the following 2 values in PROD
  private _coinbase = EthAddress.ZERO;
  private _feeRecipient = AztecAddress.ZERO;
  private lastPublishedBlock = 0;
  private state = SequencerState.STOPPED;
  private allowedInSetup: AllowedElement[] = [];
  private allowedInTeardown: AllowedElement[] = [];
  private maxBlockSizeInBytes: number = 1024 * 1024;
  private skipSubmitProofs: boolean = false;

  public readonly tracer: Tracer;

  constructor(
    private publisher: L1Publisher,
    private globalsBuilder: GlobalVariableBuilder,
    private p2pClient: P2P,
    private worldState: WorldStateSynchronizer,
    private prover: BlockProver,
    private l2BlockSource: L2BlockSource,
    private l1ToL2MessageSource: L1ToL2MessageSource,
    private publicProcessorFactory: PublicProcessorFactory,
    private txValidatorFactory: TxValidatorFactory,
    telemetry: TelemetryClient,
    config: SequencerConfig = {},
    private log = createDebugLogger('aztec:sequencer'),
  ) {
    this.updateConfig(config);
    this.tracer = telemetry.getTracer('Sequencer');
    this.log.verbose(`Initialized sequencer with ${this.minTxsPerBLock}-${this.maxTxsPerBlock} txs per block.`);
  }

  /**
   * Updates sequencer config.
   * @param config - New parameters.
   */
  public updateConfig(config: SequencerConfig) {
    if (config.transactionPollingIntervalMS !== undefined) {
      this.pollingIntervalMs = config.transactionPollingIntervalMS;
    }
    if (config.maxTxsPerBlock !== undefined) {
      this.maxTxsPerBlock = config.maxTxsPerBlock;
    }
    if (config.minTxsPerBlock !== undefined) {
      this.minTxsPerBLock = config.minTxsPerBlock;
    }
    if (config.minSecondsBetweenBlocks !== undefined) {
      this.minSecondsBetweenBlocks = config.minSecondsBetweenBlocks;
    }
    if (config.maxSecondsBetweenBlocks !== undefined) {
      this.maxSecondsBetweenBlocks = config.maxSecondsBetweenBlocks;
    }
    if (config.coinbase) {
      this._coinbase = config.coinbase;
    }
    if (config.feeRecipient) {
      this._feeRecipient = config.feeRecipient;
    }
    if (config.allowedInSetup) {
      this.allowedInSetup = config.allowedInSetup;
    }
    if (config.maxBlockSizeInBytes !== undefined) {
      this.maxBlockSizeInBytes = config.maxBlockSizeInBytes;
    }
    // TODO(#5917) remove this. it is no longer needed since we don't need to whitelist functions in teardown
    if (config.allowedInTeardown) {
      this.allowedInTeardown = config.allowedInTeardown;
    }
    // TODO(palla/prover) This flag should not be needed: the sequencer should be initialized with a blockprover
    // that does not return proofs at all (just simulates circuits), and use that to determine whether to submit
    // proofs or not.
    this.skipSubmitProofs = !!config.sequencerSkipSubmitProofs;
  }

  /**
   * Starts the sequencer and moves to IDLE state. Blocks until the initial sync is complete.
   */
  public async start() {
    await this.initialSync();

    this.runningPromise = new RunningPromise(this.work.bind(this), this.pollingIntervalMs);
    this.runningPromise.start();
    this.state = SequencerState.IDLE;
    this.log.info('Sequencer started');
  }

  /**
   * Stops the sequencer from processing txs and moves to STOPPED state.
   */
  public async stop(): Promise<void> {
    this.log.debug(`Stopping sequencer`);
    await this.runningPromise?.stop();
    this.publisher.interrupt();
    this.state = SequencerState.STOPPED;
    this.log.info('Stopped sequencer');
  }

  /**
   * Starts a previously stopped sequencer.
   */
  public restart() {
    this.log.info('Restarting sequencer');
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
        this.log.debug(`Block has been synced`);
        this.state = SequencerState.IDLE;
      }

      // Do not go forward with new block if the previous one has not been mined and processed
      if (!prevBlockSynced) {
        return;
      }

      const historicalHeader = (await this.l2BlockSource.getBlock(-1))?.header;
      const newBlockNumber =
        (historicalHeader === undefined
          ? await this.l2BlockSource.getBlockNumber()
          : Number(historicalHeader.globalVariables.blockNumber.toBigInt())) + 1;

      // Do not go forward with new block if not my turn
      if (!(await this.publisher.isItMyTurnToSubmit())) {
        this.log.debug('Not my turn to submit block');
        return;
      }

      // Compute time elapsed since the previous block
      const lastBlockTime = historicalHeader?.globalVariables.timestamp.toNumber() || 0;
      const currentTime = Math.floor(Date.now() / 1000);
      const elapsedSinceLastBlock = currentTime - lastBlockTime;

      // Do not go forward with new block if not enough time has passed since last block
      if (this.minSecondsBetweenBlocks > 0 && elapsedSinceLastBlock < this.minSecondsBetweenBlocks) {
        this.log.debug(
          `Not creating block because not enough time has passed since last block (last block at ${lastBlockTime} current time ${currentTime})`,
        );
        return;
      }

      this.state = SequencerState.WAITING_FOR_TXS;

      // Get txs to build the new block.
      const pendingTxs = this.p2pClient.getTxs('pending');

      // If we haven't hit the maxSecondsBetweenBlocks, we need to have at least minTxsPerBLock txs.
      if (pendingTxs.length < this.minTxsPerBLock) {
        if (this.skipMinTxsPerBlockCheck(elapsedSinceLastBlock)) {
          this.log.debug(
            `Creating block with only ${pendingTxs.length} txs as more than ${this.maxSecondsBetweenBlocks}s have passed since last block`,
          );
        } else {
          this.log.debug(
            `Not creating block because not enough txs in the pool (got ${pendingTxs.length} min ${this.minTxsPerBLock})`,
          );
          return;
        }
      }
      this.log.debug(`Retrieved ${pendingTxs.length} txs from P2P pool`);

      const newGlobalVariables = await this.globalsBuilder.buildGlobalVariables(
        new Fr(newBlockNumber),
        this._coinbase,
        this._feeRecipient,
      );

      // TODO: It should be responsibility of the P2P layer to validate txs before passing them on here
      const allValidTxs = await this.takeValidTxs(
        pendingTxs,
        this.txValidatorFactory.validatorForNewTxs(newGlobalVariables, this.allowedInSetup),
      );

      // TODO: We are taking the size of the tx from private-land, but we should be doing this after running
      // public functions. Only reason why we do it here now is because the public processor and orchestrator
      // are set up such that they require knowing the total number of txs in advance. Still, main reason for
      // exceeding max block size in bytes is contract class registration, which happens in private-land. This
      // may break if we start emitting lots of log data from public-land.
      const validTxs = this.takeTxsWithinMaxSize(allValidTxs);

      // Bail if we don't have enough valid txs
      if (!this.skipMinTxsPerBlockCheck(elapsedSinceLastBlock) && validTxs.length < this.minTxsPerBLock) {
        this.log.debug(
          `Not creating block because not enough valid txs loaded from the pool (got ${validTxs.length} min ${this.minTxsPerBLock})`,
        );
        return;
      }

      await this.buildBlockAndPublish(validTxs, newGlobalVariables, historicalHeader, elapsedSinceLastBlock);
    } catch (err) {
      if (BlockProofError.isBlockProofError(err)) {
        const txHashes = err.txHashes.filter(h => !h.isZero());
        this.log.warn(`Proving block failed, removing ${txHashes.length} txs from pool`);
        await this.p2pClient.deleteTxs(txHashes);
      }
      this.log.error(`Rolling back world state DB due to error assembling block`, (err as any).stack);
      // Cancel any further proving on the block
      this.prover?.cancelBlock();
      await this.worldState.getLatest().rollback();
    }
  }

  /** Whether to skip the check of min txs per block if more than maxSecondsBetweenBlocks has passed since the previous block. */
  private skipMinTxsPerBlockCheck(elapsed: number): boolean {
    return this.maxSecondsBetweenBlocks > 0 && elapsed >= this.maxSecondsBetweenBlocks;
  }

  @trackSpan('Sequencer.buildBlockAndPublish', (_validTxs, newGlobalVariables, _historicalHeader) => ({
    [Attributes.BLOCK_NUMBER]: newGlobalVariables.blockNumber.toNumber(),
  }))
  private async buildBlockAndPublish(
    validTxs: Tx[],
    newGlobalVariables: GlobalVariables,
    historicalHeader: Header | undefined,
    elapsedSinceLastBlock: number,
  ): Promise<void> {
    const workTimer = new Timer();
    this.state = SequencerState.CREATING_BLOCK;
    this.log.info(`Building block ${newGlobalVariables.blockNumber.toNumber()} with ${validTxs.length} transactions`);

    const assertBlockHeight = async () => {
      const currentBlockNumber = await this.l2BlockSource.getBlockNumber();
      if (currentBlockNumber + 1 !== newGlobalVariables.blockNumber.toNumber()) {
        throw new Error('New block was emitted while building block');
      }

      if (!(await this.publisher.isItMyTurnToSubmit())) {
        throw new Error(`Not this sequencer turn to submit block`);
      }
    };

    // Get l1 to l2 messages from the contract
    this.log.debug('Requesting L1 to L2 messages from contract');
    const l1ToL2Messages = await this.l1ToL2MessageSource.getL1ToL2Messages(newGlobalVariables.blockNumber.toBigInt());
    this.log.verbose(
      `Retrieved ${l1ToL2Messages.length} L1 to L2 messages for block ${newGlobalVariables.blockNumber.toNumber()}`,
    );

    // We create a fresh processor each time to reset any cached state (eg storage writes)
    const processor = this.publicProcessorFactory.create(historicalHeader, newGlobalVariables);

    const numRealTxs = validTxs.length;
    const blockSize = Math.max(2, numRealTxs);

    const blockBuildingTimer = new Timer();
    const blockTicket = await this.prover.startNewBlock(blockSize, newGlobalVariables, l1ToL2Messages);

    const [publicProcessorDuration, [processedTxs, failedTxs]] = await elapsed(() =>
      processor.process(validTxs, blockSize, this.prover, this.txValidatorFactory.validatorForProcessedTxs()),
    );
    if (failedTxs.length > 0) {
      const failedTxData = failedTxs.map(fail => fail.tx);
      this.log.debug(`Dropping failed txs ${Tx.getHashes(failedTxData).join(', ')}`);
      await this.p2pClient.deleteTxs(Tx.getHashes(failedTxData));
    }

    // TODO: This check should be processedTxs.length < this.minTxsPerBLock, so we don't publish a block with
    // less txs than the minimum. But that'd cause the entire block to be aborted and retried. Instead, we should
    // go back to the p2p pool and load more txs until we hit our minTxsPerBLock target. Only if there are no txs
    // we should bail.
    if (processedTxs.length === 0 && !this.skipMinTxsPerBlockCheck(elapsedSinceLastBlock)) {
      this.log.verbose('No txs processed correctly to build block. Exiting');
      this.prover.cancelBlock();
      return;
    }

    await assertBlockHeight();

    // All real transactions have been added, set the block as full and complete the proving.
    await this.prover.setBlockCompleted();

    // Here we are now waiting for the block to be proven.
    // TODO(@PhilWindle) We should probably periodically check for things like another
    // block being published before ours instead of just waiting on our block
    const result = await blockTicket.provingPromise;
    if (result.status === PROVING_STATUS.FAILURE) {
      throw new Error(`Block proving failed, reason: ${result.reason}`);
    }

    await assertBlockHeight();

    // Block is proven, now finalise and publish!
    const { block, aggregationObject, proof } = await this.prover.finaliseBlock();

    await assertBlockHeight();

    this.log.verbose(`Assembled block ${block.number}`, {
      eventName: 'l2-block-built',
      duration: workTimer.ms(),
      publicProcessDuration: publicProcessorDuration,
      rollupCircuitsDuration: blockBuildingTimer.ms(),
      ...block.getStats(),
    } satisfies L2BlockBuiltStats);

    await this.publishL2Block(block);
    this.log.info(`Submitted rollup block ${block.number} with ${processedTxs.length} transactions`);

    // Submit the proof if we have configured this sequencer to run with an actual prover.
    // This is temporary while we submit one proof per block, but will have to change once we
    // move onto proving batches of multiple blocks at a time.
    if (aggregationObject && proof && !this.skipSubmitProofs) {
      await this.publisher.submitProof(block.header, block.archive.root, aggregationObject, proof);
      this.log.info(`Submitted proof for block ${block.number}`);
    }
  }

  /**
   * Publishes the L2Block to the rollup contract.
   * @param block - The L2Block to be published.
   */
  @trackSpan('Sequencer.publishL2Block', block => ({
    [Attributes.BLOCK_NUMBER]: block.number,
  }))
  protected async publishL2Block(block: L2Block) {
    // Publishes new block to the network and awaits the tx to be mined
    this.state = SequencerState.PUBLISHING_BLOCK;
    const publishedL2Block = await this.publisher.processL2Block(block);
    if (publishedL2Block) {
      this.lastPublishedBlock = block.number;
    } else {
      throw new Error(`Failed to publish block`);
    }
  }

  protected async takeValidTxs<T extends Tx | ProcessedTx>(txs: T[], validator: TxValidator<T>): Promise<T[]> {
    const [valid, invalid] = await validator.validateTxs(txs);
    if (invalid.length > 0) {
      this.log.debug(`Dropping invalid txs from the p2p pool ${Tx.getHashes(invalid).join(', ')}`);
      await this.p2pClient.deleteTxs(Tx.getHashes(invalid));
    }

    return valid.slice(0, this.maxTxsPerBlock);
  }

  protected takeTxsWithinMaxSize(txs: Tx[]): Tx[] {
    const maxSize = this.maxBlockSizeInBytes;
    let totalSize = 0;

    const toReturn: Tx[] = [];
    for (const tx of txs) {
      const txSize = tx.getSize() - tx.clientIvcProof.clientIvcProofBuffer.length;
      if (totalSize + txSize > maxSize) {
        this.log.warn(
          `Dropping tx ${tx.getTxHash()} with estimated size ${txSize} due to exceeding ${maxSize} block size limit (currently at ${totalSize})`,
        );
        continue;
      }
      toReturn.push(tx);
      totalSize += txSize;
    }

    return toReturn;
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
    const [worldState, p2p, l2BlockSource, l1ToL2MessageSource] = syncedBlocks;
    const result = min >= this.lastPublishedBlock;
    this.log.debug(`Sync check to last published block ${this.lastPublishedBlock} ${result ? 'succeeded' : 'failed'}`, {
      worldState,
      p2p,
      l2BlockSource,
      l1ToL2MessageSource,
    });
    return result;
  }

  get coinbase(): EthAddress {
    return this._coinbase;
  }

  get feeRecipient(): AztecAddress {
    return this._feeRecipient;
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

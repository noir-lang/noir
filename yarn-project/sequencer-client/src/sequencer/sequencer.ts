import {
  type BlockAttestation,
  type L1ToL2MessageSource,
  type L2Block,
  type L2BlockSource,
  type ProcessedTx,
  Signature,
  Tx,
  type TxValidator,
} from '@aztec/circuit-types';
import { type AllowedElement, BlockProofError, PROVING_STATUS } from '@aztec/circuit-types/interfaces';
import { type L2BlockBuiltStats } from '@aztec/circuit-types/stats';
import { AztecAddress, EthAddress, type GlobalVariables, type Header, IS_DEV_NET } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { Timer, elapsed } from '@aztec/foundation/timer';
import { type P2P } from '@aztec/p2p';
import { type PublicProcessorFactory } from '@aztec/simulator';
import { Attributes, type TelemetryClient, type Tracer, trackSpan } from '@aztec/telemetry-client';
import { type ValidatorClient } from '@aztec/validator-client';
import { type WorldStateStatus, type WorldStateSynchronizer } from '@aztec/world-state';

import { type BlockBuilderFactory } from '../block_builder/index.js';
import { type GlobalVariableBuilder } from '../global_variable_builder/global_builder.js';
import { type L1Publisher } from '../publisher/l1-publisher.js';
import { type TxValidatorFactory } from '../tx_validator/tx_validator_factory.js';
import { type SequencerConfig } from './config.js';
import { SequencerMetrics } from './metrics.js';

export type ShouldProposeArgs = {
  pendingTxsCount?: number;
  validTxsCount?: number;
  processedTxsCount?: number;
};

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
  private metrics: SequencerMetrics;
  private isFlushing: boolean = false;

  constructor(
    private publisher: L1Publisher,
    private validatorClient: ValidatorClient | undefined, // During migration the validator client can be inactive
    private globalsBuilder: GlobalVariableBuilder,
    private p2pClient: P2P,
    private worldState: WorldStateSynchronizer,
    private blockBuilderFactory: BlockBuilderFactory,
    private l2BlockSource: L2BlockSource,
    private l1ToL2MessageSource: L1ToL2MessageSource,
    private publicProcessorFactory: PublicProcessorFactory,
    private txValidatorFactory: TxValidatorFactory,
    telemetry: TelemetryClient,
    private config: SequencerConfig = {},
    private log = createDebugLogger('aztec:sequencer'),
  ) {
    this.updateConfig(config);
    this.metrics = new SequencerMetrics(telemetry, () => this.state, 'Sequencer');
    this.log.verbose(`Initialized sequencer with ${this.minTxsPerBLock}-${this.maxTxsPerBlock} txs per block.`);
  }

  get tracer(): Tracer {
    return this.metrics.tracer;
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
    // TODO: Just read everything from the config object as needed instead of copying everything into local vars.
    this.config = config;
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
      // Do not go forward with new block if the previous one has not been mined and processed
      if (!prevBlockSynced) {
        this.log.debug('Previous block has not been mined and processed yet');
        return;
      }

      if (prevBlockSynced && this.state === SequencerState.PUBLISHING_BLOCK) {
        this.log.debug(`Block has been synced`);
        this.state = SequencerState.IDLE;
      }

      const chainTip = await this.l2BlockSource.getBlock(-1);
      const historicalHeader = chainTip?.header;

      const newBlockNumber =
        (historicalHeader === undefined
          ? await this.l2BlockSource.getBlockNumber()
          : Number(historicalHeader.globalVariables.blockNumber.toBigInt())) + 1;

      const chainTipArchive = chainTip?.archive.root.toBuffer();

      let slot: bigint;
      try {
        slot = await this.canProposeBlock(historicalHeader, undefined, chainTipArchive);
      } catch (err) {
        this.log.debug(`Cannot propose for block ${newBlockNumber}`);
        return;
      }

      if (!this.shouldProposeBlock(historicalHeader, {})) {
        return;
      }

      this.state = SequencerState.WAITING_FOR_TXS;

      // Get txs to build the new block.
      const pendingTxs = this.p2pClient.getTxs('pending');

      if (!this.shouldProposeBlock(historicalHeader, { pendingTxsCount: pendingTxs.length })) {
        return;
      }
      this.log.debug(`Retrieved ${pendingTxs.length} txs from P2P pool`);

      const newGlobalVariables = await this.globalsBuilder.buildGlobalVariables(
        new Fr(newBlockNumber),
        this._coinbase,
        this._feeRecipient,
        slot,
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
      if (!this.shouldProposeBlock(historicalHeader, { validTxsCount: validTxs.length })) {
        return;
      }

      await this.buildBlockAndPublish(validTxs, newGlobalVariables, historicalHeader, chainTipArchive);
    } catch (err) {
      if (BlockProofError.isBlockProofError(err)) {
        const txHashes = err.txHashes.filter(h => !h.isZero());
        this.log.warn(`Proving block failed, removing ${txHashes.length} txs from pool`);
        await this.p2pClient.deleteTxs(txHashes);
      }
      this.log.error(`Rolling back world state DB due to error assembling block`, (err as any).stack);
      await this.worldState.getLatest().rollback();
    }
  }

  /** Whether to skip the check of min txs per block if more than maxSecondsBetweenBlocks has passed since the previous block. */
  private skipMinTxsPerBlockCheck(historicalHeader: Header | undefined): boolean {
    const lastBlockTime = historicalHeader?.globalVariables.timestamp.toNumber() || 0;
    const currentTime = Math.floor(Date.now() / 1000);
    const elapsed = currentTime - lastBlockTime;

    return this.maxSecondsBetweenBlocks > 0 && elapsed >= this.maxSecondsBetweenBlocks;
  }

  async canProposeBlock(
    historicalHeader?: Header,
    globalVariables?: GlobalVariables,
    tipArchive?: Buffer,
  ): Promise<bigint> {
    // @note  In order to be able to propose a block, a few conditions must be met:
    //        - We must be caught up to the pending chain
    //          - The tip archive must match the one from the L1
    //          - The block number should match the one from the L1
    //        - If we have built a block, the block number must match the next pending block
    //        - There cannot already be a block for the slot
    //        - If we are NOT in devnet, then the active slot must match the slot number of the block
    //        - The proposer must either be free for all or specifically us.
    //
    //        Note that the ordering of these checks are NOT optimised for performance, but to resemble the ordering
    //        that is used in the Rollup contract:
    //        - _validateHeaderForSubmissionBase
    //        - _validateHeaderForSubmissionSequencerSelection
    //
    //        Also, we are logging debug messages for checks that fail to make it easier to debug, as we are usually
    //        catching the errors.

    const { proposer, slot, pendingBlockNumber, archive } = await this.publisher.getMetadataForSlotAtNextEthBlock();

    if (await this.publisher.willSimulationFail(slot)) {
      // @note  See comment in willSimulationFail for more information
      const msg = `Simulation will fail for slot ${slot}`;
      this.log.debug(msg);
      throw new Error(msg);
    }

    // If our tip of the chain is different from the tip on L1, we should not propose a block
    // @note  This will change along with the data publication changes.
    if (tipArchive && !archive.equals(tipArchive)) {
      const msg = `Tip archive does not match the one from the L1`;
      this.log.debug(msg);
      throw new Error(msg);
    }

    // Make sure I'm caught up to the pending chain
    if (
      pendingBlockNumber > 0 &&
      (historicalHeader == undefined || historicalHeader.globalVariables.blockNumber.toBigInt() != pendingBlockNumber)
    ) {
      const msg = `Not caught up to pending block ${pendingBlockNumber}`;
      this.log.debug(msg);
      throw new Error(msg);
    }

    // If I have constructed a block, make sure that the block number matches the next pending block number
    if (globalVariables) {
      if (globalVariables.blockNumber.toBigInt() !== pendingBlockNumber + 1n) {
        const msg = `Block number mismatch. Expected ${
          pendingBlockNumber + 1n
        } but got ${globalVariables.blockNumber.toBigInt()}`;
        this.log.debug(msg);
        throw new Error(msg);
      }

      const currentBlockNumber = await this.l2BlockSource.getBlockNumber();
      if (currentBlockNumber + 1 !== globalVariables.blockNumber.toNumber()) {
        this.metrics.recordCancelledBlock();
        const msg = 'New block was emitted while building block';
        this.log.debug(msg);
        throw new Error(msg);
      }
    }

    // Do not go forward if there was already a block for the slot
    if (historicalHeader && historicalHeader.globalVariables.slotNumber.toBigInt() === slot) {
      const msg = `Block already exists for slot ${slot}`;
      this.log.debug(msg);
      throw new Error(msg);
    }

    // Related to _validateHeaderForSubmissionSequencerSelection

    if (IS_DEV_NET) {
      // If we are in devnet, make sure that we are a validator
      if ((await this.publisher.getValidatorCount()) != 0n && !(await this.publisher.amIAValidator())) {
        const msg = 'Not a validator in devnet';
        this.log.debug(msg);
        throw new Error(msg);
      }
    } else {
      // If I have a constructed a block, make sure that the slot matches the current slot number
      if (globalVariables) {
        if (slot !== globalVariables.slotNumber.toBigInt()) {
          const msg = `Slot number mismatch. Expected ${slot} but got ${globalVariables.slotNumber.toBigInt()}`;
          this.log.debug(msg);
          throw new Error(msg);
        }
      }

      // Do not go forward with new block if not free for all or my turn
      if (!proposer.isZero() && !proposer.equals(await this.publisher.getSenderAddress())) {
        const msg = 'Not my turn to submit block';
        this.log.debug(msg);
        throw new Error(msg);
      }
    }

    return slot;
  }

  shouldProposeBlock(historicalHeader: Header | undefined, args: ShouldProposeArgs): boolean {
    if (this.isFlushing) {
      this.log.verbose(`Flushing all pending txs in new block`);
      return true;
    }

    if (IS_DEV_NET) {
      // Compute time elapsed since the previous block
      const lastBlockTime = historicalHeader?.globalVariables.timestamp.toNumber() || 0;
      const currentTime = Math.floor(Date.now() / 1000);
      const elapsedSinceLastBlock = currentTime - lastBlockTime;
      this.log.debug(
        `Last block mined at ${lastBlockTime} current time is ${currentTime} (elapsed ${elapsedSinceLastBlock})`,
      );

      // If we haven't hit the maxSecondsBetweenBlocks, we need to have at least minTxsPerBLock txs.
      // Do not go forward with new block if not enough time has passed since last block
      if (this.minSecondsBetweenBlocks > 0 && elapsedSinceLastBlock < this.minSecondsBetweenBlocks) {
        this.log.debug(
          `Not creating block because not enough time ${this.minSecondsBetweenBlocks} has passed since last block`,
        );
        return false;
      }
    }

    const skipCheck = this.skipMinTxsPerBlockCheck(historicalHeader);

    // If we haven't hit the maxSecondsBetweenBlocks, we need to have at least minTxsPerBLock txs.
    if (args.pendingTxsCount != undefined) {
      if (args.pendingTxsCount < this.minTxsPerBLock) {
        if (skipCheck) {
          this.log.debug(
            `Creating block with only ${args.pendingTxsCount} txs as more than ${this.maxSecondsBetweenBlocks}s have passed since last block`,
          );
        } else {
          this.log.debug(
            `Not creating block because not enough txs in the pool (got ${args.pendingTxsCount} min ${this.minTxsPerBLock})`,
          );
          return false;
        }
      }
    }

    // Bail if we don't have enough valid txs
    if (args.validTxsCount != undefined) {
      // Bail if we don't have enough valid txs
      if (!skipCheck && args.validTxsCount < this.minTxsPerBLock) {
        this.log.debug(
          `Not creating block because not enough valid txs loaded from the pool (got ${args.validTxsCount} min ${this.minTxsPerBLock})`,
        );
        return false;
      }
    }

    // TODO: This check should be processedTxs.length < this.minTxsPerBLock, so we don't publish a block with
    // less txs than the minimum. But that'd cause the entire block to be aborted and retried. Instead, we should
    // go back to the p2p pool and load more txs until we hit our minTxsPerBLock target. Only if there are no txs
    // we should bail.
    if (args.processedTxsCount != undefined) {
      if (args.processedTxsCount === 0 && !skipCheck && this.minTxsPerBLock > 0) {
        this.log.verbose('No txs processed correctly to build block. Exiting');
        return false;
      }
    }

    return true;
  }

  @trackSpan(
    'Sequencer.buildBlockAndPublish',
    (_validTxs, newGlobalVariables, _historicalHeader, _chainTipArchive) => ({
      [Attributes.BLOCK_NUMBER]: newGlobalVariables.blockNumber.toNumber(),
    }),
  )
  private async buildBlockAndPublish(
    validTxs: Tx[],
    newGlobalVariables: GlobalVariables,
    historicalHeader: Header | undefined,
    chainTipArchive: Buffer | undefined,
  ): Promise<void> {
    this.metrics.recordNewBlock(newGlobalVariables.blockNumber.toNumber(), validTxs.length);
    const workTimer = new Timer();
    this.state = SequencerState.CREATING_BLOCK;
    this.log.info(`Building block ${newGlobalVariables.blockNumber.toNumber()} with ${validTxs.length} transactions`);

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
    const blockBuilder = this.blockBuilderFactory.create(this.worldState.getLatest());
    const blockTicket = await blockBuilder.startNewBlock(blockSize, newGlobalVariables, l1ToL2Messages);

    const [publicProcessorDuration, [processedTxs, failedTxs]] = await elapsed(() =>
      processor.process(validTxs, blockSize, blockBuilder, this.txValidatorFactory.validatorForProcessedTxs()),
    );
    if (failedTxs.length > 0) {
      const failedTxData = failedTxs.map(fail => fail.tx);
      this.log.debug(`Dropping failed txs ${Tx.getHashes(failedTxData).join(', ')}`);
      await this.p2pClient.deleteTxs(Tx.getHashes(failedTxData));
    }

    await this.canProposeBlock(historicalHeader, newGlobalVariables, chainTipArchive);
    if (
      !this.shouldProposeBlock(historicalHeader, {
        validTxsCount: validTxs.length,
        processedTxsCount: processedTxs.length,
      })
    ) {
      blockBuilder.cancelBlock();
      return;
    }

    // All real transactions have been added, set the block as full and complete the proving.
    await blockBuilder.setBlockCompleted();

    // Here we are now waiting for the block to be proven (using simulated[fake] proofs).
    // TODO(@PhilWindle) We should probably periodically check for things like another
    // block being published before ours instead of just waiting on our block
    const result = await blockTicket.provingPromise;
    if (result.status === PROVING_STATUS.FAILURE) {
      throw new Error(`Block proving failed, reason: ${result.reason}`);
    }

    // Block is ready, now finalise
    const { block } = await blockBuilder.finaliseBlock();

    await this.canProposeBlock(historicalHeader, newGlobalVariables, chainTipArchive);

    const workDuration = workTimer.ms();
    this.log.verbose(
      `Assembled block ${block.number} (txEffectsHash: ${block.header.contentCommitment.txsEffectsHash.toString(
        'hex',
      )})`,
      {
        eventName: 'l2-block-built',
        duration: workDuration,
        publicProcessDuration: publicProcessorDuration,
        rollupCircuitsDuration: blockBuildingTimer.ms(),
        ...block.getStats(),
      } satisfies L2BlockBuiltStats,
    );

    if (this.isFlushing) {
      this.log.verbose(`Flushing completed`);
    }

    this.isFlushing = false;
    const attestations = await this.collectAttestations(block);
    await this.canProposeBlock(historicalHeader, newGlobalVariables, chainTipArchive);

    try {
      await this.publishL2Block(block, attestations);
      this.metrics.recordPublishedBlock(workDuration);
      this.log.info(
        `Submitted rollup block ${block.number} with ${
          processedTxs.length
        } transactions duration=${workDuration}ms (Submitter: ${await this.publisher.getSenderAddress()})`,
      );
    } catch (err) {
      this.metrics.recordFailedBlock();
      throw err;
    }
  }

  /** Forces the sequencer to bypass all time and tx count checks for the next block and build anyway. */
  public flush() {
    this.isFlushing = true;
  }

  protected async collectAttestations(block: L2Block): Promise<Signature[] | undefined> {
    // @todo  This should collect attestations properly and fix the ordering of them to make sense
    //        the current implementation is a PLACEHOLDER and should be nuked from orbit.
    //        It is assuming that there will only be ONE (1) validator, so only one attestation
    //        is needed.
    // @note  This is quite a sin, but I'm committing war crimes in this code already.
    //            _ ._  _ , _ ._
    //          (_ ' ( `  )_  .__)
    //       ( (  (    )   `)  ) _)
    //      (__ (_   (_ . _) _) ,__)
    //           `~~`\ ' . /`~~`
    //                ;   ;
    //                /   \
    //  _____________/_ __ \_____________
    if (IS_DEV_NET || !this.validatorClient) {
      return undefined;
    }

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/7962): inefficient to have a round trip in here - this should be cached
    const committee = await this.publisher.getCurrentEpochCommittee();

    if (committee.length === 0) {
      return undefined;
    }

    const numberOfRequiredAttestations = Math.floor((committee.length * 2) / 3) + 1;

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/7974): we do not have transaction[] lists in the block for now
    // Dont do anything with the proposals for now - just collect them

    const proposal = await this.validatorClient.createBlockProposal(block.header, block.archive.root, []);

    this.state = SequencerState.PUBLISHING_BLOCK_TO_PEERS;
    this.validatorClient.broadcastBlockProposal(proposal);

    this.state = SequencerState.WAITING_FOR_ATTESTATIONS;
    const attestations = await this.validatorClient.collectAttestations(proposal, numberOfRequiredAttestations);

    // note: the smart contract requires that the signatures are provided in the order of the committee
    return await orderAttestations(attestations, committee);
  }

  /**
   * Publishes the L2Block to the rollup contract.
   * @param block - The L2Block to be published.
   */
  @trackSpan('Sequencer.publishL2Block', block => ({
    [Attributes.BLOCK_NUMBER]: block.number,
  }))
  protected async publishL2Block(block: L2Block, attestations?: Signature[]) {
    // Publishes new block to the network and awaits the tx to be mined
    this.state = SequencerState.PUBLISHING_BLOCK;

    const publishedL2Block = await this.publisher.processL2Block(block, attestations);
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
   * Publishing blocks to validator peers. Will move to WAITING_FOR_ATTESTATIONS.
   */
  PUBLISHING_BLOCK_TO_PEERS,
  /**
   * The block has been published to peers, and we are waiting for attestations. Will move to PUBLISHING_CONTRACT_DATA.
   */
  WAITING_FOR_ATTESTATIONS,
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

/** Order Attestations
 *
 * Returns attestation signatures in the order of a series of provided ethereum addresses
 * The rollup smart contract expects attestations to appear in the order of the committee
 *
 * @todo: perform this logic within the memory attestation store instead?
 */
async function orderAttestations(attestations: BlockAttestation[], orderAddresses: EthAddress[]): Promise<Signature[]> {
  // Create a map of sender addresses to BlockAttestations
  const attestationMap = new Map<string, BlockAttestation>();

  for (const attestation of attestations) {
    const sender = await attestation.getSender();
    if (sender) {
      attestationMap.set(sender.toString(), attestation);
    }
  }

  // Create the ordered array based on the orderAddresses, else return an empty signature
  const orderedAttestations = orderAddresses.map(address => {
    const addressString = address.toString();
    return attestationMap.get(addressString)?.signature || Signature.empty();
  });

  return orderedAttestations;
}

import { P2P } from '@aztec/p2p';
import {
  MerkleTreeId,
  ContractData,
  ContractPublicData,
  PrivateTx,
  PublicTx,
  Tx,
  UnverifiedData,
  isPrivateTx,
} from '@aztec/types';
import { WorldStateStatus, WorldStateSynchroniser } from '@aztec/world-state';
import times from 'lodash.times';
import { BlockBuilder } from '../block_builder/index.js';
import { L1Publisher } from '../publisher/l1-publisher.js';
import { ceilPowerOfTwo } from '../utils.js';
import { SequencerConfig } from './config.js';
import { ProcessedTx } from './processed_tx.js';
import { PublicProcessor } from './public_processor.js';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { createDebugLogger } from '@aztec/foundation/log';
import { Fr } from '@aztec/foundation/fields';
import { NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';

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

      if (processedTxs.length === 0) {
        this.log('No txs processed correctly to build block. Exiting');
        return;
      }

      // Get l1 to l2 messages from the contract
      this.log('Requesting L1 to L2 messages from contract');
      const l1ToL2Messages = this.takeL1ToL2MessagesFromContract();
      this.log('Successfully retrieved L1 to L2 messages from contract');

      // Build the new block by running the rollup circuits
      this.log(`Assembling block with txs ${processedTxs.map(tx => tx.hash).join(', ')}`);
      const block = await this.buildBlock(processedTxs, l1ToL2Messages);
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

      // Publishes new unverified data & contract data for private txs to the network and awaits the tx to be mined
      this.state = SequencerState.PUBLISHING_UNVERIFIED_DATA;
      const unverifiedData = UnverifiedData.join(validTxs.filter(isPrivateTx).map(tx => tx.unverifiedData));
      const newContractData = validTxs
        .filter(isPrivateTx)
        .map(tx => {
          // Currently can only have 1 new contract per tx
          const newContract = tx.data?.end.newContracts[0];
          if (newContract && tx.newContractPublicFunctions?.length) {
            return new ContractPublicData(
              new ContractData(newContract.contractAddress, newContract.portalContractAddress),
              tx.newContractPublicFunctions,
            );
          }
        })
        .filter((cd): cd is Exclude<typeof cd, undefined> => cd !== undefined);

      const publishedUnverifiedData = await this.publisher.processUnverifiedData(block.number, unverifiedData);
      if (publishedUnverifiedData) {
        this.log(`Successfully published unverifiedData for block ${block.number}`);
      } else {
        this.log(`Failed to publish unverifiedData for block ${block.number}`);
      }

      const publishedContractData = await this.publisher.processNewContractData(block.number, newContractData);
      if (publishedContractData) {
        this.log(`Successfully published new contract data for block ${block.number}`);
      } else if (!publishedContractData && newContractData.length) {
        this.log(`Failed to publish new contract data for block ${block.number}`);
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

  /**
   * Pads the set of txs to a power of two and assembles a block by calling the block builder.
   * @param txs - Processed txs to include in the next block.
   * @param newL1ToL2Messages - L1 to L2 messages to be part of the block.
   * @returns The new block.
   */
  protected async buildBlock(txs: ProcessedTx[], newL1ToL2Messages: Fr[]) {
    // Pad the txs array with empty txs to be a power of two, at least 4
    const txsTargetSize = Math.max(ceilPowerOfTwo(txs.length), 4);
    const emptyTxCount = txsTargetSize - txs.length;

    const allTxs = [
      ...txs,
      ...(await Promise.all(times(emptyTxCount, () => this.publicProcessor.makeEmptyProcessedTx()))),
    ];
    const [block] = await this.blockBuilder.buildL2Block(this.lastBlockNumber + 1, allTxs, newL1ToL2Messages);
    return block;
  }

  /**
   * Checks on chain messages inbox and selects messages to inlcude within the next rollup block.
   * TODO: This is a stubbed method.
   * @returns An array of L1 to L2 messages.
   */
  protected takeL1ToL2MessagesFromContract(): Fr[] {
    return new Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n));
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
   * Creating a new L2 block. Includes processing public function calls and running rollup circuits. Will move to PUBLISHING_BLOCK.
   */
  CREATING_BLOCK,
  /**
   * Sending the tx to L1 with the L2 block data and awaiting it to be mined. Will move to PUBLISHING_UNVERIFIED_DATA.
   */
  PUBLISHING_BLOCK,
  /**
   * Sending the tx to L1 with unverified data and awaiting it to be mined. Will move back to IDLE once finished.
   */
  PUBLISHING_UNVERIFIED_DATA,
  /**
   * Sequencer is stopped and not processing any txs from the pool.
   */
  STOPPED,
}

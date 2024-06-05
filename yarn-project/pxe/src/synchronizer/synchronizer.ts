import { type AztecNode, type L2Block, L2BlockL2Logs, MerkleTreeId, type TxHash } from '@aztec/circuit-types';
import { type NoteProcessorCaughtUpStats } from '@aztec/circuit-types/stats';
import { type AztecAddress, type Fr, INITIAL_L2_BLOCK_NUM, type PublicKey } from '@aztec/circuits.js';
import { type SerialQueue } from '@aztec/foundation/fifo';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { type KeyStore } from '@aztec/key-store';

import { type DeferredNoteDao } from '../database/deferred_note_dao.js';
import { type IncomingNoteDao } from '../database/incoming_note_dao.js';
import { type PxeDatabase } from '../database/index.js';
import { NoteProcessor } from '../note_processor/index.js';

/**
 * The Synchronizer class manages the synchronization of note processors and interacts with the Aztec node
 * to obtain encrypted logs, blocks, and other necessary information for the accounts.
 * It provides methods to start or stop the synchronization process, add new accounts, retrieve account
 * details, and fetch transactions by hash. The Synchronizer ensures that it maintains the note processors
 * in sync with the blockchain while handling retries and errors gracefully.
 */
export class Synchronizer {
  private runningPromise?: RunningPromise;
  private noteProcessors: NoteProcessor[] = [];
  private running = false;
  private initialSyncBlockNumber = INITIAL_L2_BLOCK_NUM - 1;
  private log: DebugLogger;
  private noteProcessorsToCatchUp: NoteProcessor[] = [];

  constructor(private node: AztecNode, private db: PxeDatabase, private jobQueue: SerialQueue, logSuffix = '') {
    this.log = createDebugLogger(logSuffix ? `aztec:pxe_synchronizer_${logSuffix}` : 'aztec:pxe_synchronizer');
  }

  /**
   * Starts the synchronization process by fetching encrypted logs and blocks from a specified position.
   * Continuously processes the fetched data for all note processors until stopped. If there is no data
   * available, it retries after a specified interval.
   *
   * @param limit - The maximum number of encrypted, unencrypted logs and blocks to fetch in each iteration.
   * @param retryInterval - The time interval (in ms) to wait before retrying if no data is available.
   */
  public async start(limit = 1, retryInterval = 1000) {
    if (this.running) {
      return;
    }
    this.running = true;

    await this.jobQueue.put(() => this.initialSync());
    this.log.info('Initial sync complete');
    this.runningPromise = new RunningPromise(() => this.sync(limit), retryInterval);
    this.runningPromise.start();
    this.log.debug('Started loop');
  }

  protected async initialSync() {
    // fast forward to the latest block
    const latestHeader = await this.node.getHeader();
    this.initialSyncBlockNumber = Number(latestHeader.globalVariables.blockNumber.toBigInt());
    await this.db.setHeader(latestHeader);
  }

  /**
   * Fetches encrypted logs and blocks from the Aztec node and processes them for all note processors.
   * If needed, catches up note processors that are lagging behind the main sync, e.g. because we just added a new account.
   *
   * Uses the job queue to ensure that
   * - sync does not overlap with pxe simulations.
   * - one sync is running at a time.
   *
   * @param limit - The maximum number of encrypted, unencrypted logs and blocks to fetch in each iteration.
   * @returns a promise that resolves when the sync is complete
   */
  protected sync(limit: number) {
    return this.jobQueue.put(async () => {
      let moreWork = true;
      // keep external this.running flag to interrupt greedy sync
      while (moreWork && this.running) {
        if (this.noteProcessorsToCatchUp.length > 0) {
          // There is a note processor that needs to catch up. We hijack the main loop to catch up the note processor.
          moreWork = await this.workNoteProcessorCatchUp(limit);
        } else {
          // No note processor needs to catch up. We continue with the normal flow.
          moreWork = await this.work(limit);
        }
      }
    });
  }

  /**
   * Fetches encrypted logs and blocks from the Aztec node and processes them for all note processors.
   *
   * @param limit - The maximum number of encrypted, unencrypted logs and blocks to fetch in each iteration.
   * @returns true if there could be more work, false if we're caught up or there was an error.
   */
  protected async work(limit = 1): Promise<boolean> {
    const from = this.getSynchedBlockNumber() + 1;
    try {
      const blocks = await this.node.getBlocks(from, limit);
      if (blocks.length === 0) {
        return false;
      }

      const noteEncryptedLogs = blocks.flatMap(block => block.body.noteEncryptedLogs);

      // Update latest tree roots from the most recent block
      const latestBlock = blocks[blocks.length - 1];
      await this.setHeaderFromBlock(latestBlock);

      const logCount = L2BlockL2Logs.getTotalLogCount(noteEncryptedLogs);
      this.log.debug(
        `Forwarding ${logCount} encrypted logs and blocks to ${this.noteProcessors.length} note processors`,
      );
      for (const noteProcessor of this.noteProcessors) {
        // TODO(#6830): pass in only the blocks
        await noteProcessor.process(blocks, noteEncryptedLogs);
      }
      return true;
    } catch (err) {
      this.log.error(`Error in synchronizer work`, err);
      return false;
    }
  }

  /**
   * Catch up note processors that are lagging behind the main sync.
   * e.g. because we just added a new account.
   *
   * @param limit - the maximum number of encrypted, unencrypted logs and blocks to fetch in each iteration.
   * @returns true if there could be more work, false if there was an error which allows a retry with delay.
   */
  protected async workNoteProcessorCatchUp(limit = 1): Promise<boolean> {
    const toBlockNumber = this.getSynchedBlockNumber();

    // filter out note processors that are already caught up
    // and sort them by the block number they are lagging behind in ascending order
    const noteProcessorsToCatchUp: NoteProcessor[] = [];

    this.noteProcessorsToCatchUp.forEach(noteProcessor => {
      if (noteProcessor.status.syncedToBlock >= toBlockNumber) {
        // Note processor is ahead of main sync, nothing to do
        this.noteProcessors.push(noteProcessor);
      } else {
        noteProcessorsToCatchUp.push(noteProcessor);
      }
    });

    this.noteProcessorsToCatchUp = noteProcessorsToCatchUp;

    if (!this.noteProcessorsToCatchUp.length) {
      // No note processors to catch up, nothing to do here,
      // but we return true to continue with the normal flow.
      return true;
    }

    // create a copy so that:
    // 1. we can modify the original array while iterating over it
    // 2. we don't need to serialize insertions into the array
    const catchUpGroup = this.noteProcessorsToCatchUp
      .slice()
      // sort by the block number they are lagging behind
      .sort((a, b) => a.status.syncedToBlock - b.status.syncedToBlock);

    // grab the note processor that is lagging behind the most
    const from = catchUpGroup[0].status.syncedToBlock + 1;
    // Ensuring that the note processor does not sync further than the main sync.
    limit = Math.min(limit, toBlockNumber - from + 1);
    // this.log(`Catching up ${catchUpGroup.length} note processors by up to ${limit} blocks starting at block ${from}`);

    if (limit < 1) {
      throw new Error(`Unexpected limit ${limit} for note processor catch up`);
    }

    try {
      const blocks = await this.node.getBlocks(from, limit);
      if (!blocks.length) {
        // This should never happen because this function should only be called when the note processor is lagging
        // behind main sync.
        throw new Error('No blocks in processor catch up mode');
      }

      const noteEncryptedLogs = blocks.flatMap(block => block.body.noteEncryptedLogs);

      const logCount = L2BlockL2Logs.getTotalLogCount(noteEncryptedLogs);
      this.log.debug(`Forwarding ${logCount} encrypted logs and blocks to note processors in catch up mode`);

      for (const noteProcessor of catchUpGroup) {
        // find the index of the first block that the note processor is not yet synced to
        const index = blocks.findIndex(block => block.number > noteProcessor.status.syncedToBlock);
        if (index === -1) {
          // Due to the limit, we might not have fetched a new enough block for the note processor.
          // And since the group is sorted, we break as soon as we find a note processor
          // that needs blocks newer than the newest block we fetched.
          break;
        }

        this.log.debug(
          `Catching up note processor ${noteProcessor.account.toString()} by processing ${
            blocks.length - index
          } blocks`,
        );
        await noteProcessor.process(blocks.slice(index), noteEncryptedLogs.slice(index));

        if (noteProcessor.status.syncedToBlock === toBlockNumber) {
          // Note processor caught up, move it to `noteProcessors` from `noteProcessorsToCatchUp`.
          this.log.debug(`Note processor for ${noteProcessor.account.toString()} has caught up`, {
            eventName: 'note-processor-caught-up',
            account: noteProcessor.account.toString(),
            duration: noteProcessor.timer.ms(),
            dbSize: this.db.estimateSize(),
            ...noteProcessor.stats,
          } satisfies NoteProcessorCaughtUpStats);

          this.noteProcessorsToCatchUp = this.noteProcessorsToCatchUp.filter(
            np => !np.account.equals(noteProcessor.account),
          );
          this.noteProcessors.push(noteProcessor);
        }
      }

      return true; // could be more work, immediately continue syncing
    } catch (err) {
      this.log.error(`Error in synchronizer workNoteProcessorCatchUp`, err);
      return false;
    }
  }

  private async setHeaderFromBlock(latestBlock: L2Block) {
    if (latestBlock.number < this.initialSyncBlockNumber) {
      return;
    }

    await this.db.setHeader(latestBlock.header);
  }

  /**
   * Stops the synchronizer gracefully, interrupting any ongoing sleep and waiting for the current
   * iteration to complete before setting the running state to false. Once stopped, the synchronizer
   * will no longer process blocks or encrypted logs and must be restarted using the start method.
   *
   * @returns A promise that resolves when the synchronizer has successfully stopped.
   */
  public async stop() {
    this.running = false;
    await this.runningPromise?.stop();
    this.log.info('Stopped');
  }

  /**
   * Add a new account to the Synchronizer with the specified private key.
   * Creates a NoteProcessor instance for the account and pushes it into the noteProcessors array.
   * The method resolves immediately after pushing the new note processor.
   *
   * @param publicKey - The public key for the account.
   * @param keyStore - The key store.
   * @param startingBlock - The block where to start scanning for notes for this accounts.
   * @returns A promise that resolves once the account is added to the Synchronizer.
   */
  public async addAccount(account: AztecAddress, keyStore: KeyStore, startingBlock: number) {
    const predicate = (x: NoteProcessor) => x.account.equals(account);
    const processor = this.noteProcessors.find(predicate) ?? this.noteProcessorsToCatchUp.find(predicate);
    if (processor) {
      return;
    }

    this.noteProcessorsToCatchUp.push(await NoteProcessor.create(account, keyStore, this.db, this.node, startingBlock));
  }

  /**
   * Checks if the specified account is synchronized.
   * @param account - The aztec address for which to query the sync status.
   * @returns True if the account is fully synched, false otherwise.
   * @remarks Checks whether all the notes from all the blocks have been processed. If it is not the case, the
   *          retrieved information from contracts might be old/stale (e.g. old token balance).
   * @throws If checking a sync status of account which is not registered.
   */
  public async isAccountStateSynchronized(account: AztecAddress) {
    const completeAddress = await this.db.getCompleteAddress(account);
    if (!completeAddress) {
      throw new Error(`Checking if account is synched is not possible for ${account} because it is not registered.`);
    }
    const findByAccountAddress = (x: NoteProcessor) => x.account.equals(completeAddress.address);
    const processor =
      this.noteProcessors.find(findByAccountAddress) ?? this.noteProcessorsToCatchUp.find(findByAccountAddress);
    if (!processor) {
      throw new Error(
        `Checking if account is synched is not possible for ${account} because it is only registered as a recipient.`,
      );
    }
    return await processor.isSynchronized();
  }

  private getSynchedBlockNumber() {
    return this.db.getBlockNumber() ?? this.initialSyncBlockNumber;
  }

  /**
   * Checks whether all the blocks were processed (tree roots updated, txs updated with block info, etc.).
   * @returns True if there are no outstanding blocks to be synched.
   * @remarks This indicates that blocks and transactions are synched even if notes are not.
   * @remarks Compares local block number with the block number from aztec node.
   */
  public async isGlobalStateSynchronized() {
    const latest = await this.node.getBlockNumber();
    return latest <= this.getSynchedBlockNumber();
  }

  /**
   * Returns the latest block that has been synchronized by the synchronizer and each account.
   * @returns The latest block synchronized for blocks, and the latest block synched for notes for each public key being tracked.
   */
  public getSyncStatus() {
    const lastBlockNumber = this.getSynchedBlockNumber();
    return {
      blocks: lastBlockNumber,
      notes: Object.fromEntries(this.noteProcessors.map(n => [n.account.toString(), n.status.syncedToBlock])),
    };
  }

  /**
   * Returns the note processor stats.
   * @returns The note processor stats for notes for each public key being tracked.
   */
  public getSyncStats() {
    return Object.fromEntries(this.noteProcessors.map(n => [n.account.toString(), n.stats]));
  }

  /**
   * Retry decoding any deferred notes for the specified contract address.
   * @param contractAddress - the contract address that has just been added
   */
  public reprocessDeferredNotesForContract(contractAddress: AztecAddress): Promise<void> {
    return this.jobQueue.put(() => this.#reprocessDeferredNotesForContract(contractAddress));
  }

  async #reprocessDeferredNotesForContract(contractAddress: AztecAddress): Promise<void> {
    const deferredNotes = await this.db.getDeferredNotesByContract(contractAddress);

    // group deferred notes by txHash to properly deal with possible duplicates
    const txHashToDeferredNotes: Map<TxHash, DeferredNoteDao[]> = new Map();
    for (const note of deferredNotes) {
      const notesForTx = txHashToDeferredNotes.get(note.txHash) ?? [];
      notesForTx.push(note);
      txHashToDeferredNotes.set(note.txHash, notesForTx);
    }

    // keep track of decoded notes
    const incomingNotes: IncomingNoteDao[] = [];
    // now process each txHash
    for (const deferredNotes of txHashToDeferredNotes.values()) {
      // to be safe, try each note processor in case the deferred notes are for different accounts.
      for (const processor of this.noteProcessors) {
        const notes = await processor.decodeDeferredNotes(deferredNotes);
        incomingNotes.push(...notes);
      }
    }

    // now drop the deferred notes, and add the decoded notes
    await this.db.removeDeferredNotesByContract(contractAddress);
    await this.db.addNotes(incomingNotes, []);

    incomingNotes.forEach(noteDao => {
      this.log.debug(
        `Decoded deferred note for contract ${noteDao.contractAddress} at slot ${
          noteDao.storageSlot
        } with nullifier ${noteDao.siloedNullifier.toString()}`,
      );
    });

    // now group the decoded incoming notes by public key
    const publicKeyToNotes: Map<PublicKey, IncomingNoteDao[]> = new Map();
    for (const noteDao of incomingNotes) {
      const notesForPublicKey = publicKeyToNotes.get(noteDao.ivpkM) ?? [];
      notesForPublicKey.push(noteDao);
      publicKeyToNotes.set(noteDao.ivpkM, notesForPublicKey);
    }

    // now for each group, look for the nullifiers in the nullifier tree
    for (const [publicKey, notes] of publicKeyToNotes.entries()) {
      const nullifiers = notes.map(n => n.siloedNullifier);
      const relevantNullifiers: Fr[] = [];
      for (const nullifier of nullifiers) {
        // NOTE: this leaks information about the nullifiers I'm interested in to the node.
        const found = await this.node.findLeafIndex('latest', MerkleTreeId.NULLIFIER_TREE, nullifier);
        if (found) {
          relevantNullifiers.push(nullifier);
        }
      }
      await this.db.removeNullifiedNotes(relevantNullifiers, publicKey);
    }
  }
}

import { AztecAddress, BlockHeader, Fr, PublicKey } from '@aztec/circuits.js';
import { computeGlobalsHash } from '@aztec/circuits.js/abis';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { InterruptibleSleep } from '@aztec/foundation/sleep';
import { AztecNode, INITIAL_L2_BLOCK_NUM, KeyStore, L2BlockContext, L2BlockL2Logs, LogType } from '@aztec/types';
import { NoteProcessorCaughtUpStats } from '@aztec/types/stats';

import { PxeDatabase } from '../database/index.js';
import { NoteProcessor } from '../note_processor/index.js';

/**
 * The Synchronizer class manages the synchronization of note processors and interacts with the Aztec node
 * to obtain encrypted logs, blocks, and other necessary information for the accounts.
 * It provides methods to start or stop the synchronization process, add new accounts, retrieve account
 * details, and fetch transactions by hash. The Synchronizer ensures that it maintains the note processors
 * in sync with the blockchain while handling retries and errors gracefully.
 */
export class Synchronizer {
  private runningPromise?: Promise<void>;
  private noteProcessors: NoteProcessor[] = [];
  private interruptibleSleep = new InterruptibleSleep();
  private running = false;
  private initialSyncBlockNumber = 0;
  private synchedToBlock = 0;
  private log: DebugLogger;
  private noteProcessorsToCatchUp: NoteProcessor[] = [];

  constructor(private node: AztecNode, private db: PxeDatabase, logSuffix = '') {
    this.log = createDebugLogger(logSuffix ? `aztec:pxe_synchronizer_${logSuffix}` : 'aztec:pxe_synchronizer');
  }

  /**
   * Starts the synchronization process by fetching encrypted logs and blocks from a specified position.
   * Continuously processes the fetched data for all note processors until stopped. If there is no data
   * available, it retries after a specified interval.
   *
   * @param from - The starting position for fetching encrypted logs and blocks.
   * @param limit - The maximum number of encrypted, unencrypted logs and blocks to fetch in each iteration.
   * @param retryInterval - The time interval (in ms) to wait before retrying if no data is available.
   */
  public async start(from = INITIAL_L2_BLOCK_NUM, limit = 1, retryInterval = 1000) {
    if (this.running) {
      return;
    }
    this.running = true;

    if (from < this.synchedToBlock + 1) {
      throw new Error(`From block ${from} is smaller than the currently synched block ${this.synchedToBlock}`);
    }
    this.synchedToBlock = from - 1;

    await this.initialSync();

    const run = async () => {
      while (this.running) {
        if (this.noteProcessorsToCatchUp.length > 0) {
          // There is a note processor that needs to catch up. We hijack the main loop to catch up the note processor.
          await this.workNoteProcessorCatchUp(limit, retryInterval);
        } else {
          // No note processor needs to catch up. We continue with the normal flow.
          await this.work(limit, retryInterval);
        }
      }
    };

    this.runningPromise = run();
    this.log('Started');
  }

  protected async initialSync() {
    const [blockNumber, blockHeader] = await Promise.all([this.node.getBlockNumber(), this.node.getBlockHeader()]);
    this.initialSyncBlockNumber = blockNumber;
    this.synchedToBlock = this.initialSyncBlockNumber;
    await this.db.setBlockHeader(blockHeader);
  }

  protected async work(limit = 1, retryInterval = 1000): Promise<void> {
    const from = this.synchedToBlock + 1;
    try {
      let encryptedLogs = await this.node.getLogs(from, limit, LogType.ENCRYPTED);
      if (!encryptedLogs.length) {
        await this.interruptibleSleep.sleep(retryInterval);
        return;
      }

      let unencryptedLogs = await this.node.getLogs(from, limit, LogType.UNENCRYPTED);
      if (!unencryptedLogs.length) {
        await this.interruptibleSleep.sleep(retryInterval);
        return;
      }

      // Note: If less than `limit` encrypted logs is returned, then we fetch only that number of blocks.
      const blocks = await this.node.getBlocks(from, encryptedLogs.length);
      if (!blocks.length) {
        await this.interruptibleSleep.sleep(retryInterval);
        return;
      }

      if (blocks.length !== encryptedLogs.length) {
        // "Trim" the encrypted logs to match the number of blocks.
        encryptedLogs = encryptedLogs.slice(0, blocks.length);
      }

      if (blocks.length !== unencryptedLogs.length) {
        // "Trim" the unencrypted logs to match the number of blocks.
        unencryptedLogs = unencryptedLogs.slice(0, blocks.length);
      }

      // attach logs to blocks
      blocks.forEach((block, i) => {
        block.attachLogs(encryptedLogs[i], LogType.ENCRYPTED);
        block.attachLogs(unencryptedLogs[i], LogType.UNENCRYPTED);
      });

      // Wrap blocks in block contexts.
      const blockContexts = blocks.map(block => new L2BlockContext(block));

      // Update latest tree roots from the most recent block
      const latestBlock = blockContexts[blockContexts.length - 1];
      await this.setBlockDataFromBlock(latestBlock);

      const logCount = L2BlockL2Logs.getTotalLogCount(encryptedLogs);
      this.log(`Forwarding ${logCount} encrypted logs and blocks to ${this.noteProcessors.length} note processors`);
      for (const noteProcessor of this.noteProcessors) {
        await noteProcessor.process(blockContexts, encryptedLogs);
      }

      this.synchedToBlock = latestBlock.block.number;
    } catch (err) {
      this.log.error(`Error in synchronizer work`, err);
      await this.interruptibleSleep.sleep(retryInterval);
    }
  }

  protected async workNoteProcessorCatchUp(limit = 1, retryInterval = 1000): Promise<void> {
    const noteProcessor = this.noteProcessorsToCatchUp[0];
    if (noteProcessor.status.syncedToBlock === this.synchedToBlock) {
      // Note processor already synched, nothing to do
      this.noteProcessorsToCatchUp.shift();
      this.noteProcessors.push(noteProcessor);
      return;
    }

    const from = noteProcessor.status.syncedToBlock + 1;
    // Ensuring that the note processor does not sync further than the main sync.
    limit = Math.min(limit, this.synchedToBlock - from + 1);

    if (limit < 1) {
      throw new Error(`Unexpected limit ${limit} for note processor catch up`);
    }

    try {
      let encryptedLogs = await this.node.getLogs(from, limit, LogType.ENCRYPTED);
      if (!encryptedLogs.length) {
        // This should never happen because this function should only be called when the note processor is lagging
        // behind main sync.
        throw new Error('No encrypted logs in processor catch up mode');
      }

      // Note: If less than `limit` encrypted logs is returned, then we fetch only that number of blocks.
      const blocks = await this.node.getBlocks(from, encryptedLogs.length);
      if (!blocks.length) {
        // This should never happen because this function should only be called when the note processor is lagging
        // behind main sync.
        throw new Error('No blocks in processor catch up mode');
      }

      if (blocks.length !== encryptedLogs.length) {
        // "Trim" the encrypted logs to match the number of blocks.
        encryptedLogs = encryptedLogs.slice(0, blocks.length);
      }

      const blockContexts = blocks.map(block => new L2BlockContext(block));

      const logCount = L2BlockL2Logs.getTotalLogCount(encryptedLogs);
      this.log(`Forwarding ${logCount} encrypted logs and blocks to note processor in catch up mode`);
      await noteProcessor.process(blockContexts, encryptedLogs);

      if (noteProcessor.status.syncedToBlock === this.synchedToBlock) {
        // Note processor caught up, move it to `noteProcessors` from `noteProcessorsToCatchUp`.
        this.log(`Note processor for ${noteProcessor.publicKey.toString()} has caught up`, {
          eventName: 'note-processor-caught-up',
          publicKey: noteProcessor.publicKey.toString(),
          duration: noteProcessor.timer.ms(),
          dbSize: this.db.estimateSize(),
          ...noteProcessor.stats,
        } satisfies NoteProcessorCaughtUpStats);
        this.noteProcessorsToCatchUp.shift();
        this.noteProcessors.push(noteProcessor);
      }
    } catch (err) {
      this.log.error(`Error in synchronizer workNoteProcessorCatchUp`, err);
      await this.interruptibleSleep.sleep(retryInterval);
    }
  }

  private async setBlockDataFromBlock(latestBlock: L2BlockContext) {
    const { block } = latestBlock;
    if (block.number < this.initialSyncBlockNumber) {
      return;
    }

    const globalsHash = computeGlobalsHash(latestBlock.block.globalVariables);
    const blockHeader = new BlockHeader(
      block.endNoteHashTreeSnapshot.root,
      block.endNullifierTreeSnapshot.root,
      block.endContractTreeSnapshot.root,
      block.endL1ToL2MessagesTreeSnapshot.root,
      block.endArchiveSnapshot.root,
      Fr.ZERO, // todo: private kernel vk tree root
      block.endPublicDataTreeRoot,
      globalsHash,
    );

    await this.db.setBlockHeader(blockHeader);
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
    this.interruptibleSleep.interrupt();
    await this.runningPromise;
    this.log('Stopped');
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
  public addAccount(publicKey: PublicKey, keyStore: KeyStore, startingBlock: number) {
    const predicate = (x: NoteProcessor) => x.publicKey.equals(publicKey);
    const processor = this.noteProcessors.find(predicate) ?? this.noteProcessorsToCatchUp.find(predicate);
    if (processor) {
      return;
    }

    this.noteProcessorsToCatchUp.push(new NoteProcessor(publicKey, keyStore, this.db, this.node, startingBlock));
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
    const findByPublicKey = (x: NoteProcessor) => x.publicKey.equals(completeAddress.publicKey);
    const processor = this.noteProcessors.find(findByPublicKey) ?? this.noteProcessorsToCatchUp.find(findByPublicKey);
    if (!processor) {
      throw new Error(
        `Checking if account is synched is not possible for ${account} because it is only registered as a recipient.`,
      );
    }
    return await processor.isSynchronized();
  }

  /**
   * Checks whether all the blocks were processed (tree roots updated, txs updated with block info, etc.).
   * @returns True if there are no outstanding blocks to be synched.
   * @remarks This indicates that blocks and transactions are synched even if notes are not.
   * @remarks Compares local block number with the block number from aztec node.
   */
  public async isGlobalStateSynchronized() {
    const latest = await this.node.getBlockNumber();
    return latest <= this.synchedToBlock;
  }

  /**
   * Returns the latest block that has been synchronized by the synchronizer and each account.
   * @returns The latest block synchronized for blocks, and the latest block synched for notes for each public key being tracked.
   */
  public getSyncStatus() {
    return {
      blocks: this.synchedToBlock,
      notes: Object.fromEntries(this.noteProcessors.map(n => [n.publicKey.toString(), n.status.syncedToBlock])),
    };
  }
}
